#!/usr/bin/env python3
"""Disassemble a Mega Bomberman ROM into asm/ that GNU as rebuilds exactly.

    tools/disasm.py VERSION            (reads baserom.VERSION.bin)

Code is found by tracing from the vector table (tools/m68k.py decodes):
calls, branches, and the indirect forms whose targets are fixed in the code
(docs/LAYOUT.md, "Finding code"):

  - `jmp`/`jsr (d8,pc,Xn)` into a table of 4-byte entries (`bra.w x`, or
    two 2-byte instructions such as `nop; rts`);
  - a code pointer loaded into an address register (`lea x,an`,
    `movea.l #x,an`) and called or jumped through in the same block;
  - a long stored into a RAM slot (`move.l #x,(slot)`) where a slot is any
    RAM long some block loads into an address register and jumps through;
  - code.VERSION.txt: addresses a runtime trace ran that nothing here
    reaches (tools/coverage.py --write), one per line.

data.VERSION.txt lists ranges a trace read as data; speculation leaves
them alone.

Then speculation (Disasm.speculate_all): candidate addresses the code or
the data between code names, and unclaimed addresses after a jump or
return, each traced in a journal and kept only if everything it reaches
decodes cleanly in the code area (docs/LAYOUT.md, "Finding code").

Everything else is data.  Every word is accounted for either way, so the
build matches no matter how much is found; what is found decides how much
is symbolic (and so can move, SHIFT=1) and readable.

Output (one version at a time; .version names it):
  asm/header.s        the vector table and the cartridge header
  asm/NNNNNN.s        the ROM from NNNNNN on, cut where long data starts
  asm/link.ld         the objects in ROM order, and `. = ADDR;` where
                      the data starts after the zeros that follow the code
  asm/symbols.txt     every global label: address, name, kind
"""

import argparse
import collections
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import m68k  # noqa: E402

TOP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HEADER_END = 0x200
# a data run at least this long starts a new file
DATA_SPLIT = 0x100
ZERO_RUN = 32        # zero runs this long become .space
DATA_MIN_INSNS = 6   # see Disasm.speculate_all
CALL_TABLE_BIAS = 2  # leading slots of a call table no index reaches
STRIDE_MAX = 0x20    # longest record a pointer table extends over
SHIFT_FUNC_PAD = 2   # shift_pad's size (include/macros.inc)
# Absolute operands that are a table's address minus the smallest index
# (a biased base): {version: {operand: table}}, written `table-off`.
BIASED = {
    # func_00F06C (eu): `lea (x).l,a0; move.b (0,a0,d0.w),...` with d0 an
    # animation frame, 0x86-0x93: the table is D_00EFB8 (us)
    'us': {0xEF32: 0xEFB8, 0xEF33: 0xEFB8},
    'eu': {0xEF2C: 0xEFB2, 0xEF2D: 0xEFB2},
}

# Bytecode scripts (scene and object scripts) whose longs are pointers.
# A script is read a byte at a time through a pointer in RAM (`slot`) by
# `readers` (a byte, a word and a long; each reads bytes one by one); its
# roots are the long immediates stored into the slot.  `func_015FAE` runs
# one op per call: the op's byte, then its operands as below
# (SCRIPT_OPS, checked against what the handlers in its table at `ops`
# read).  What an op 0x0F call reads after it is found from the
# callee's code (Disasm.script_reads).
SCRIPTS = {
    'us': [dict(slot=0xFFAD5E, readers=(0x15F54, 0x15F62, 0x15F80),
                ops=0x16004)],
    'eu': [dict(slot=0xFFAD5E, readers=(0x15F4E, 0x15F5C, 0x15F7A),
                ops=0x15FFE)],
}
# func_015FAE's ops: x a pad byte (the operand after it is even),
# b a byte, w a word, l a long (a pointer where it can be one: op 0x0E's
# goes to (0x4,a6)), c a function called (it may read on), s a script
# address (op 0x11: n times back to it, then on).  0x00 waits a frame;
# 0x80 and up yields with carry, and the walk stops there.
SCRIPT_OPS = {
    0x00: 'b', 0x01: 'xw', 0x02: 'b', 0x03: 'b', 0x04: 'xw', 0x05: 'b',
    0x06: 'b', 0x07: 'b', 0x08: 'b', 0x09: 'xw', 0x0A: 'b', 0x0B: 'b',
    0x0C: 'xw', 0x0D: 'b', 0x0E: 'xl', 0x0F: 'xc', 0x10: 'b', 0x11: 'bs',
}
SCRIPT_SIZE = {'x': 1, 'b': 1, 'w': 2, 'l': 4, 'c': 4, 's': 4}


# Tables of records holding pointers, by what the code does with each
# field: `table` is a table of record pointers (it ends where the first
# record it points at starts), a record is longs of these kinds: d data
# (a pointer), c a table of code pointers the code calls through, f a
# function.  func_001030 copies the record (D_00A04A, by (0x953A).w) to
# 0x9E78-0x9E87: func_005336 calls through 0x9E80 (index & 0x3F),
# func_000E98 jumps to 0x9E84; nothing reads 0x9E78 or 0x9E7C by name.
RECORDS = {
    'us': [dict(table=0xA04A, fields='ddcf')],
    'eu': [dict(table=0xA044, fields='ddcf')],
}

class Unknown(Exception):
    """What a function reads from a script can't be told from its code."""


# Reads past a table into the code after it (the game's own bugs) that a
# SHIFT build must leave alone: no pad in [lo, hi].  The same addresses in
# us and eu (before 0x7C4A).
READS_PAST = [
    # func_003F1A's .L003F70: `move.b (D_003FA5,pc,d0.w),d0` with d0 a byte
    # of D_003FA2 not sign-extended: 0xFE (-2) reads 0x40A3, an opcode
    # byte, and an enemy moves by it
    (0x3FA6, 0x40A3),
]


def is_ram(a):
    return a >= 0xFF0000


def pointer_imm(ins):
    """Is ins's long immediate used as an address?  `movea.l #x,an`, and
    `move.l #x` into memory or onto the stack (a pushed return address);
    `move.l #x,dn` or `cmpi.l #x` may as well be a number."""
    if ins.mn == 'movea.l':
        return True
    return ins.mn == 'move.l' and ins.ops[1].kind != 'dn'


def address_imm(ins):
    """Is ins's long immediate an address when it falls in the ROM?
    pointer_imm's cases, and a base added to an index (`addi.l #x,d0`,
    `adda.l #x,a0`).  `move.l #x,dn`, `cmpi.l`, `andi.l` and the like
    are numbers."""
    return pointer_imm(ins) or ins.mn in ('addi.l', 'adda.l')


def gaps(rom, n=0x10000):
    """Where nothing points: runs of n or more zero bytes (between the
    code and the data) and the zeros the ROM ends with.  Shorter runs
    inside the data are alignment, and a block after one may start with
    zeros (`0xBD000`, tiles), so pointers into those are fine."""
    out = []
    a = 0
    while a < len(rom):
        if rom[a]:
            a += 1
            continue
        b = a
        while b < len(rom) and rom[b] == 0:
            b += 1
        if b - a >= n or b == len(rom):
            out.append((a, b))
        a = b
    return out


def code_limit(rom):
    """End of the code area: the end of the first 16K block, after one
    with code, that has no rts in it.  Speculation stays below it."""
    B = 0x4000
    for blk in range(0, len(rom), B):
        if not any(rom[x:x + 2] == b'\x4e\x75'
                   for x in range(blk, blk + B, 2)):
            return blk
    return len(rom)


class Disasm:
    def __init__(self, rom, extra=(), data=(), longs=()):
        self.rom = rom
        # ROM addresses a trace read as longs (longs.VERSION.txt)
        self.longs = set(longs)
        self.gaps = gaps(rom)
        self.ptrs = {}           # addr of a pointer long in data -> target
        self.biased = {}         # BIASED[version]
        self.end = len(rom)
        self.insns = {}          # addr -> Insn
        self.owner = {}          # byte addr -> insn start, for code bytes
        self.funcs = set()       # called or entry
        self.branch = set()      # branch targets
        self.jtbl = set()        # jump table starts
        self.jtbl_entries = set()
        # bases of `movea.l (tbl,pc,dn),an; jsr (an)` tables of longs
        self.call_tables = set()
        self.table_longs = set() # their entries (find_pointers' seeds)
        # ROM longs the code reads off a `lea` base (Disasm.look)
        self.read_longs = set()
        self.scripts = []        # SCRIPTS[version]
        self.records = []        # RECORDS[version]
        self.version = None
        self.vmap = None         # tools/vermap.py's, outside us
        # instructions whose long immediate is a number although it
        # equals an address: the other version has the same number where
        # that address moved (tools/vermap.py, numbers())
        self.numbers = set()
        # longs in data that moved with the code between the versions
        # (tools/vermap.py, moved_longs()): pointers, like a traced read
        self.moved = set()
        self.script_ptrs = {}    # pointer longs in scripts -> target
        self.script_bytes = set()  # every byte a script walk covered
        self.dlabels = set()     # data addresses the code names
        self.conflicts = []
        self.slots = set()       # RAM longs jumped through
        self.slot_stores = collections.defaultdict(set)
        self.extra = set(extra)
        # bytes known to be data (read as data in a trace): speculation
        # must not claim them
        self.known_data = bytearray(len(rom))
        for lo, hi in data:
            for k in range(lo, min(hi, len(rom))):
                self.known_data[k] = 1
        self.code_limit = code_limit(rom)
        self.journal = None      # list of undo steps while speculating
        self.failed = False
        self.probable = set()    # roots accepted by speculation

    def in_rom(self, a):
        return HEADER_END <= a < self.end

    # every change goes through these, so speculation can be undone
    def _add(self, st, v):
        if v not in st:
            st.add(v)
            if self.journal is not None:
                self.journal.append((st.discard, v))

    def _put(self, d, k, v):
        assert k not in d
        d[k] = v
        if self.journal is not None:
            self.journal.append((d.pop, k))

    def _fail(self, a):
        if self.journal is not None:
            self.failed = True
        else:
            self.conflicts.append(a)

    # -- tracing -----------------------------------------------------------

    def trace_all(self):
        vec = struct.unpack('>64I', self.rom[:256])
        roots = set(v for v in vec[1:] if self.in_rom(v) and not v & 1)
        self.funcs |= roots
        roots |= self.extra
        self.funcs |= self.extra
        todo = list(roots)
        while True:
            while todo:
                self.trace(todo.pop(), todo)
            # stores into slots found after the stores were traced
            more = set()
            for slot in self.slots:
                more |= self.slot_stores.get(slot, set())
            more = [a for a in more if a not in self.insns and
                    self.in_rom(a) and not a & 1]
            if not more:
                break
            self.funcs |= set(more)
            todo = more

    def speculate(self, a, min_insns=1):
        """Trace from a as code if everything it reaches decodes cleanly
        and stays clear of known code; else leave no trace of it."""
        if a in self.insns or a in self.owner or a & 1 or \
                not HEADER_END <= a < self.code_limit:
            return False
        self.journal = []
        self.failed = False
        todo = [a]
        while todo and not self.failed:
            self.trace(todo.pop(), todo)
        # slots this closure stores into, if they are known slots
        journal, self.journal = self.journal, None
        added = sum(1 for undo, k in journal if undo == self.insns.pop)
        if added < min_insns:
            self.failed = True
        if self.failed:
            for undo, k in reversed(journal):
                undo(k)
            self.failed = False
            return False
        self.funcs.add(a)
        self.probable.add(a)
        return True

    def candidates(self):
        """Addresses the code names that might be code: long immediates
        and absolute operands."""
        out = set()
        for ins in self.insns.values():
            for t, kind in ins.refs():
                if kind in ('absl', 'absw', 'pcdisp'):
                    out.add(t)
                elif kind == 'imm' and pointer_imm(ins):
                    out.add(t)
        return sorted(out)

    def data_candidates(self):
        """Longs in the data between code (pointer tables) that point into
        the code area."""
        out = set()
        for ins in self.insns.values():
            for t, kind in ins.refs():
                if kind == 'imm' and not pointer_imm(ins):
                    out.add(t)
        lim = self.code_limit
        for a in range(HEADER_END, lim - 3, 2):
            if a in self.owner:
                continue
            v = struct.unpack_from('>I', self.rom, a)[0]
            if HEADER_END <= v < lim and not v & 1:
                out.add(v)
        return sorted(out)

    def speculate_all(self):
        """Candidates the code names first, to a fixpoint; then one pass
        of pointers found in data, which must reach DATA_MIN_INSNS
        instructions (a short one is likelier the tail of a real
        instruction that looks like code: `4CDF 7070` holds `moveq; rts`);
        then, if those found nothing, one of the unclaimed addresses right
        after a jump or return, with the same minimum.  Repeat while any
        finds something."""
        while True:
            while self._speculate_named():
                pass
            n = 0
            for a in self.data_candidates():
                if self.speculate(a, DATA_MIN_INSNS):
                    n += 1
            if n:
                continue
            for a in self.gap_starts():
                if self.speculate(a, DATA_MIN_INSNS):
                    n += 1
            if n:
                continue
            named = self.named_data()
            for a in self.gap_starts():
                if self.fill_gap(a, named):
                    n += 1
            if not n:
                break

    def named_data(self):
        """ROM addresses the code names other than as a branch or call
        target: what it reads (absolute, pc-relative, table bases)."""
        out = set()
        for ins in self.insns.values():
            for t, kind in ins.refs():
                if kind not in ('label', 'imm'):
                    out.add(t)
            out.update(self.disp_refs(ins))
        return out

    def fill_gap(self, a, named):
        """Take the gap from a (right after a jump or a return) to the
        next known code as code, however short, if it is closures that
        fill it exactly: each ends in a jump or a return, the next starts
        right after, the last ends where the known code starts, nothing
        reaches outside the gap and the code names nothing in it as data
        (named_data).  Dead functions nothing names (0x868E: two siblings
        of func_00867E) are found this way; data seldom ends with a
        return right at code."""
        if a in self.owner or a & 1 or not HEADER_END <= a < self.code_limit:
            return False
        e = a
        while e < self.code_limit and e not in self.owner:
            e += 1
        if e >= self.code_limit or e not in self.insns or \
                any(k in named for k in range(a, e)):
            return False
        self.journal = []
        self.failed = False
        starts = []
        b = a
        while b < e and not self.failed:
            starts.append(b)
            todo = [b]
            while todo and not self.failed:
                self.trace(todo.pop(), todo)
            if self.failed:
                break
            # the next unclaimed byte must follow a jump or a return
            while b < e and b in self.owner:
                b += 1
            if b < e:
                p = self.owner.get(b - 2)
                if p is None or p + self.insns[p].size != b or \
                        self.insns[p].flow not in ('jump', 'ret'):
                    self.failed = True
        journal, self.journal = self.journal, None
        if not self.failed:
            added = [k for undo, k in journal if undo == self.insns.pop]
            p = self.owner.get(e - 1)
            # nothing outside the gap; it ends in a jump or a return (not
            # a few bytes that run into the code after)
            if any(not a <= k < e for k in added) or p is None or \
                    p + self.insns[p].size != e or \
                    self.insns[p].flow not in ('jump', 'ret'):
                self.failed = True
        if self.failed:
            for undo, k in reversed(journal):
                undo(k)
            self.failed = False
            return False
        for b in starts:
            self.funcs.add(b)
            self.probable.add(b)
        return True

    def gap_starts(self):
        """Unclaimed addresses in the code area right after a jump or a
        return: where a function nothing visibly names would start."""
        out = []
        for a, ins in self.insns.items():
            b = a + ins.size
            if ins.flow in ('jump', 'ret') and b not in self.owner and \
                    b < self.code_limit:
                out.append(b)
        return sorted(out)

    def record_fields(self):
        """Walk RECORDS: their pointer longs go to table_longs (seeds
        of find_pointers), their call tables to call_tables.  Returns
        the functions they name."""
        def long_at(a):
            if a + 4 > self.code_limit or \
                    any(a + k in self.owner for k in range(4)):
                return None
            v = struct.unpack_from('>I', self.rom, a)[0]
            if v & 1 or not HEADER_END <= v < self.end:
                return None
            return v
        out = []
        for rt in self.records:
            t = rt['table']
            end = self.code_limit
            a = t
            while a < end:
                r = long_at(a)
                if r is None:
                    break
                self.table_longs.add(a)
                if r > t:
                    end = min(end, r)
                for k, c in enumerate(rt['fields']):
                    v = long_at(r + 4 * k)
                    if v is None:
                        continue
                    self.table_longs.add(r + 4 * k)
                    if c == 'c':
                        self._add(self.call_tables, v)
                    elif c == 'f' and v not in self.insns:
                        out.append(v)
                a += 4
        return out

    def call_table(self, t):
        """The entries of a table of code pointers at base t, as
        (address, target).  The base may be biased (func_015FAE's table:
        opcode 0 never indexes it, so its first slot is the code before):
        up to CALL_TABLE_BIAS longs that can't be a code address are
        skipped.  The table ends at what isn't one, at an instruction, or
        where the first of its targets after it starts."""
        def code_addr(a):
            if a + 4 > self.code_limit or \
                    any(a + k in self.owner for k in range(4)):
                return None
            v = struct.unpack_from('>I', self.rom, a)[0]
            if v & 1 or not HEADER_END <= v < self.code_limit:
                return None
            return v
        a = t
        for _ in range(CALL_TABLE_BIAS):
            if code_addr(a) is not None:
                break
            a += 4
        out = []
        end = self.code_limit
        while a < end:
            v = code_addr(a)
            if v is None:
                break
            out.append((a, v))
            if v > a:
                end = min(end, v)
            a += 4
        return out

    def script_reads(self, f, readers, memo, stack=()):
        """The sizes of what the function at f reads from a script, in
        order: every path from f to its return reads the same, a loop
        that comes back reads nothing more, calls are followed.  Raises
        Unknown otherwise (an indirect call, code not found)."""
        if f in readers:
            return [readers[f]]
        if f in memo:
            return memo[f]
        if f in stack or len(stack) > 32:
            raise Unknown(f)
        stack += (f,)

        def walk(a, seen):
            out = []
            while True:
                if a in seen:
                    return None      # back to a loop: nothing new
                seen.add(a)
                ins = self.insns.get(a)
                if ins is None:
                    raise Unknown(a)
                if ins.flow in ('call', 'jump') and ins.target is None:
                    raise Unknown(a)
                if ins.flow == 'call':
                    out += self.script_reads(ins.target, readers, memo, stack)
                elif ins.flow == 'ret':
                    return out
                elif ins.flow == 'jump':
                    if ins.target in self.funcs:
                        return out + self.script_reads(ins.target, readers,
                                                       memo, stack)
                    a = ins.target
                    continue
                elif ins.flow == 'branch':
                    x = walk(ins.target, set(seen))
                    y = walk(a + ins.size, set(seen))
                    if x is None and y is None:
                        return None
                    if x is not None and y is not None and x != y:
                        raise Unknown(a)
                    return out + (y if x is None else x)
                a += ins.size
        r = walk(f, set())
        if r is None:
            raise Unknown(f)
        memo[f] = r
        return r

    def check_script_ops(self, sc, readers, memo):
        """SCRIPT_OPS against the handlers in the interpreter's table:
        each reads its operands (a call reads a long, then calls)."""
        for op, fmt in SCRIPT_OPS.items():
            a = sc['ops'] + 4 * op
            if not 0 < op < 0x11 or a not in self.ptrs and \
                    a not in self.table_longs:
                continue
            h = struct.unpack_from('>I', self.rom, a)[0]
            if h not in self.insns or fmt.endswith('c'):
                continue         # not code yet; a call reads on
            want = [SCRIPT_SIZE[c] for c in fmt]
            try:
                got = self.script_reads(h, readers, memo)
            except Unknown:
                got = None
            if got != want:
                raise ValueError('script op %02X: handler %06X reads %s, '
                                 'SCRIPT_OPS says %s' % (op, h, got, want))

    def walk_scripts(self):
        """Walk every script from its roots (SCRIPTS): its pointer longs
        go to script_ptrs.  Returns the functions scripts call that
        aren't code yet; a walk stops at such a call (what it reads is
        unknown until it is code), at an op it doesn't know, at a yield
        (0x80 and up) and where it has been."""
        self.script_ptrs = {}
        self.script_bytes = set()
        calls = set()
        for sc in self.scripts:
            readers = dict(zip(sc['readers'], (1, 2, 4)))
            memo = {}
            self.check_script_ops(sc, readers, memo)
            todo = sorted(self.slot_stores.get(sc['slot'], ()))
            seen = set()
            while todo:
                a = todo.pop()
                while self.in_rom(a) and a not in seen and \
                        a not in self.owner:
                    seen.add(a)
                    fmt = SCRIPT_OPS.get(self.rom[a])
                    if fmt is None:
                        break
                    b = a + 1
                    more = []
                    for c in fmt:
                        if c in 'lcs':
                            t = struct.unpack_from('>I', self.rom, b)[0]
                            if c == 's':
                                todo.append(t)
                            if c == 'c':
                                if t not in self.insns:
                                    calls.add(t)
                                    more = None
                                else:
                                    try:
                                        more = self.script_reads(
                                            t, readers, memo)
                                    except Unknown:
                                        more = None
                            if c != 'l' or self.target_ok(t):
                                self.script_ptrs[b] = t
                        b += SCRIPT_SIZE[c]
                    if more is None:
                        self.script_bytes.update(range(a, b))
                        break
                    for k in more:
                        if k == 4:
                            t = struct.unpack_from('>I', self.rom, b)[0]
                            if self.target_ok(t):
                                self.script_ptrs[b] = t
                        b += k
                    self.script_bytes.update(range(a, b))
                    a = b
        return sorted(a for a in calls
                      if HEADER_END <= a < self.code_limit and not a & 1)

    def _speculate_named(self):
        n = 0
        for a in self.candidates():
            if self.speculate(a):
                n += 1
        # tables of records (RECORDS)
        for a in self.record_fields():
            if self.speculate(a):
                n += 1
        # tables the code calls through: each entry code, else the table
        # ends before it
        for t in sorted(self.call_tables):
            ents = self.call_table(t)
            if ents and ents[0][0] != t and \
                    self.owner.get(t) not in (None, t):
                # a biased base inside an instruction (`lea (x).l` of
                # 0x22B8's table): written table-off (BIASED)
                self.biased.setdefault(t, ents[0][0])
            for a, v in ents:
                if v not in self.insns:
                    if not self.speculate(v):
                        break
                    n += 1
                if v in self.insns:
                    self.table_longs.add(a)
        # functions scripts call
        for a in self.walk_scripts():
            if self.speculate(a):
                n += 1
        # slot stores the new code made
        more = set()
        for slot in self.slots:
            more |= self.slot_stores.get(slot, set())
        for a in sorted(more):
            if a not in self.insns and self.speculate(a):
                n += 1
        return n

    def trace(self, a, todo):
        block = []
        while True:
            if a in self.insns:
                return
            if self.failed:
                return
            if a in self.owner or not self.in_rom(a) or a & 1:
                if a in self.owner or self.journal is not None:
                    self._fail(a)
                return
            ins = m68k.decode(self.rom, a)
            if ins is None or any(a + k in self.owner
                                  for k in range(ins.size)):
                self._fail(a)
                return
            if self.journal is not None and (
                    any(self.known_data[a + k] for k in range(ins.size)) or
                    ins.raw or ins.mn in ('illegal', 'reset', 'stop') or
                    self.rom[a:a + 4] == bytes(4) or
                    ins.mn.startswith('trap') or a >= self.code_limit):
                self._fail(a)
                return
            self._put(self.insns, a, ins)
            for k in range(ins.size):
                self._put(self.owner, a + k, a)
            block.append(ins)
            self.look(ins, block, todo)
            t = ins.target
            if t is not None and self.journal is not None and \
                    (t & 1 or not HEADER_END <= t < self.code_limit):
                self._fail(a)
                return
            if t is not None and self.in_rom(t):
                self._add(self.funcs if ins.flow == 'call' else self.branch,
                          t)
                todo.append(t)
            if ins.flow in ('jump', 'ret'):
                return
            if ins.flow == 'branch':
                block = []
            a += ins.size

    def look(self, ins, block, todo):
        """Indirect flow and code pointers at ins, given its block so far."""
        mn = ins.mn
        if mn in ('jmp', 'jsr') and ins.ops[0].kind == 'pcindex':
            self.jump_table(ins.ops[0].addr, todo)
            return
        if mn in ('jmp', 'jsr') and ins.ops[0].kind == 'ind':
            reg = ins.ops[0].reg
            # what loaded this register last in the block?
            for p in reversed(block[:-1]):
                ops = p.ops
                if len(ops) == 2 and ops[1].kind == 'an' and \
                        ops[1].reg == reg:
                    src = ops[0]
                    if p.mn == 'lea' and src.kind in ('absl', 'absw',
                                                      'pcdisp'):
                        self.code_ptr(src.addr, todo)
                    elif p.mn == 'movea.l' and src.kind == 'imm':
                        self.code_ptr(src.val, todo)
                    elif p.mn == 'movea.l' and src.kind in ('absw', 'absl') \
                            and is_ram(src.addr):
                        self._add(self.slots, src.addr)
                    elif p.mn == 'movea.l' and src.kind == 'pcindex':
                        self._add(self.call_tables, src.addr)
                    elif p.mn == 'movea.l' and src.kind == 'index':
                        # `lea (x).l,a0; movea.l (d,a0,dn.w),a0;
                        # jsr (a0)`: a table at x+d
                        t = self.lea_base(block[:block.index(p)], src.reg)
                        if t is not None:
                            self._add(self.call_tables, t + src.val)
                    break
            return
        if mn in ('move.l', 'movea.l') and \
                ins.ops[0].kind in ('ind', 'disp'):
            # a long read off a `lea (x).l` base in the block
            src = ins.ops[0]
            t = self.lea_base(block[:-1], src.reg)
            if t is not None and self.in_rom(t):
                self._add(self.read_longs,
                          t + (src.val if src.kind == 'disp' else 0))
        if mn == 'move.l' and ins.ops[0].kind == 'imm' and \
                ins.ops[1].kind in ('absw', 'absl') and \
                is_ram(ins.ops[1].addr):
            st = self.slot_stores[ins.ops[1].addr]
            self._add(st, ins.ops[0].val)

    @staticmethod
    def lea_base(block, reg):
        """The ROM address the last `lea (x).l`/`(x,pc)` in block loaded
        into address register reg, if nothing else wrote it since."""
        for p in reversed(block):
            ops = p.ops
            if any(o.kind in ('postinc', 'predec') and o.reg == reg
                   for o in ops):
                return None      # moved on from x
            if len(ops) == 2 and ops[1].kind == 'an' and ops[1].reg == reg:
                if p.mn == 'lea' and ops[0].kind in ('absl', 'absw',
                                                     'pcdisp'):
                    return ops[0].addr
                return None
        return None

    def code_ptr(self, a, todo):
        if self.in_rom(a) and not a & 1:
            self._add(self.funcs, a)
            todo.append(a)

    def jump_table(self, t, todo):
        if not self.in_rom(t) or t & 1:
            return
        self._add(self.jtbl, t)
        a = t
        while self.in_rom(a) and a not in self.insns and \
                (a == t or a not in self.funcs):
            ins = m68k.decode(self.rom, a)
            if ins is None or ins.raw:
                break
            if ins.size == 4 and ins.flow == 'jump':
                pass
            elif ins.size == 2:
                second = m68k.decode(self.rom, a + 2)
                if second is None or second.size != 2 or \
                        second.flow not in ('jump', 'ret'):
                    break
            else:
                break
            self._add(self.branch, a)
            self._add(self.jtbl_entries, a)
            todo.append(a)
            a += 4

    # -- references --------------------------------------------------------

    def target_ok(self, t):
        """Could t be what a pointer points at?  In the header (its
        fields), at an instruction start or in data in the code area, or
        in the data; not in a gap of zeros, not inside an instruction."""
        if not 0x100 <= t < self.end:
            return False
        if any(lo < t < hi for lo, hi in self.gaps):
            return False
        o = self.owner.get(t)
        return o is None or o == t

    def find_pointers(self):
        """Pointer longs in data: longs a trace read (longs.VERSION.txt)
        whose value target_ok() accepts, odd values only into the data
        (text), and from each of those, its neighbours 4 bytes on either
        side while they hold such values too (the rest of the table,
        entries no trace picked; also with the strides between seeds, for
        records).  A long must lie wholly in data and not overlap another;
        one no trace read must point into the data or at something the
        code or another pointer names."""
        def ok(a, ev):
            if a < HEADER_END or a + 4 > self.end or \
                    any(a + k in self.owner for k in range(4)):
                return False
            if any(b in self.ptrs for b in (a - 2, a, a + 2)) or \
                    (a & 1 and not ev):
                return False
            t = struct.unpack_from('>I', self.rom, a)[0]
            if t & 1 and t < self.code_limit:
                return False
            # a neighbour no trace read, into the code area, must point
            # at something already named: palette scripts hold `0000 6D00`
            if not ev and t < self.code_limit and t not in stops and \
                    t not in self.funcs and t not in self.branch:
                return False
            return self.target_ok(t)
        self.walk_scripts()
        self.ptrs.update(self.script_ptrs)
        stops = set()
        # longs the code reads off a `lea` base, where they hold a
        # function's entry (a number may equal anything else)
        read = set(a for a in self.read_longs if a + 4 <= self.end and
                   struct.unpack_from('>I', self.rom, a)[0] in self.funcs)
        seeds = sorted(a for a in self.longs | self.table_longs | read |
                       self.moved if ok(a, True))
        for a in seeds:
            self.ptrs[a] = struct.unpack_from('>I', self.rom, a)[0]
        # the strides: 4, and the distance from each seed to the others
        # up to STRIDE_MAX after it (records: 0x3704 is a table of
        # 10-byte records with two pointers each, 4 and 6 bytes apart)
        strides = collections.defaultdict(set)
        for i, a in enumerate(seeds):
            strides[a].add(4)
            for b in seeds[i + 1:]:
                if b - a > STRIDE_MAX:
                    break
                if not (b - a) & 1 and b - a >= 4:
                    strides[a].add(b - a)
                    strides[b].add(b - a)
        # what the code or a pointer names starts something else: a
        # table doesn't run over it (the data after a table is often
        # what it points at)
        stops |= set(self.ptrs.values())
        for ins in self.insns.values():
            for t, kind in ins.refs():
                if kind in ('absw', 'absl', 'pcdisp', 'pcindex') or (
                        kind == 'imm' and self.imm_pointer(ins, t)):
                    stops.add(t)

        def crosses(a, b):
            if b > a:
                return any(x in stops for x in range(a + 1, b + 4))
            return any(x in stops for x in range(b + 1, a + 1))
        for a in seeds:
            if a in self.moved and a not in self.longs | self.table_longs:
                continue         # one pointer: dead code holds them too
            for s in sorted(strides[a]):
                for step in (s, -s):
                    p, b = a, a + step
                    while ok(b, False) and not crosses(p, b):
                        t = struct.unpack_from('>I', self.rom, b)[0]
                        self.ptrs[b] = t
                        stops.add(t)
                        p, b = b, b + step
        # a target that starts a table too, from its start 4 bytes on:
        # where the pointer's neighbours point at tables of pointers
        # (D_00B534's seventh, D_00B728, no trace read), or where it
        # carries on a table, two pointers before it (a record at 0x37E4
        # points into D_00FEB0's middle, which no longer runs over it)
        more = True
        while more:
            more = False
            for a, t in sorted(self.ptrs.items()):
                if t in self.ptrs or not ok(t, False):
                    continue
                sib = [self.ptrs.get(a + k) for k in (-4, 4)]
                if not (t - 4 in self.ptrs and t - 8 in self.ptrs) and \
                        not any(x is not None and x in self.ptrs for x in sib):
                    continue
                p, b = None, t
                while ok(b, False) and (p is None or not crosses(p, b)):
                    v = struct.unpack_from('>I', self.rom, b)[0]
                    self.ptrs[b] = v
                    stops.add(v)
                    p, b = b, b + 4
                    more = True

    def disp_pointer(self, v):
        """Is a displacement v off an address register a ROM address?
        Where it lands on data in the code area a trace read
        (`move.b (0x17A6,a0),d0`: a0 is an index); the code's struct
        offsets (0x200, 0x1C2E) land in code."""
        return HEADER_END <= v < self.code_limit and \
            bool(self.known_data[v]) and v not in self.owner

    def disp_refs(self, ins):
        return [o.val for o in ins.ops
                if o.kind == 'disp' and o.val > 0 and
                self.disp_pointer(o.val)]

    def imm_pointer(self, ins, v):
        """Is the long immediate v of ins an address?  Where ins uses it
        as one (address_imm), or wherever it is a function's entry
        (`move.l #func,d0` hands a callback over in a register)."""
        if ins.addr in self.numbers:
            return False
        if address_imm(ins):
            return self.target_ok(v)
        return v in self.funcs and v in self.insns and \
            ins.mn not in ('andi.l', 'ori.l', 'eori.l')

    def collect_refs(self):
        """Data labels: ROM addresses named by code that aren't code, and
        what pointers (immediates, longs in data) point at."""
        self.absref = set()      # named other than pc-relatively
        self.find_pointers()
        targets = set(self.ptrs.values())
        for a, ins in self.insns.items():
            for t, kind in ins.refs():
                if kind == 'imm':
                    if self.imm_pointer(ins, t):
                        targets.add(t)
                        self.absref.add(t)
                    continue
                if kind != 'label':
                    self.absref.add(t)
                if self.in_rom(t) and t not in self.insns and \
                        kind != 'label':
                    self.dlabels.add(t)
        for ins in self.insns.values():
            targets.update(self.disp_refs(ins))
        targets.update(self.biased.values())
        for t in targets:
            self.absref.add(t)
            if t < HEADER_END:
                continue         # header+off
            if t not in self.insns:
                self.dlabels.add(t)
            elif t not in self.branch:
                self.funcs.add(t)    # an entry reached through a pointer
        # a label inside a pointer long can't be placed: that long stays
        # bytes
        while True:
            bad = [a for a in self.ptrs if any(
                a + k in self.dlabels for k in (1, 2, 3))]
            if not bad:
                break
            for a in bad:
                del self.ptrs[a]

    def tag(self, a):
        """What a label at a is called after: its address in us (the
        version map, tools/vermap.py), or its own with the version
        appended where us has nothing there (`D_00A07E_eu`)."""
        if self.vmap is None:
            return '%06X' % a
        r = self.vmap.to_ref(a)
        if r is None:
            return '%06X_%s' % (a, self.version)
        return '%06X' % r

    def labels(self):
        """addr -> (name, kind) for every label emitted."""
        out = {}
        t = self.tag
        for a in self.dlabels:
            if a in self.owner and self.owner[a] != a:
                continue         # inside an instruction: named relative
            out[a] = ('D_' + t(a), 'data')
        for a in self.jtbl:
            out[a] = ('jtbl_' + t(a), 'jtbl')
        for a in self.branch:
            if a in self.insns:
                if a in self.absref or a in self.extra:
                    out[a] = ('loc_' + t(a), 'loc')
                else:
                    out[a] = ('.L' + t(a), 'branch')
        for a in self.funcs:
            if a in self.insns:
                out[a] = ('func_' + t(a), 'func')
        return out


def zero_run(rom, a, end):
    n = 0
    while a + n < end and rom[a + n] == 0:
        n += 1
    return n


class Writer:
    def __init__(self, d, rom, version):
        self.d = d
        self.rom = rom
        self.version = version
        self.labels = d.labels()
        self.sorted_labels = sorted(self.labels)
        # what near() may count from: .L labels are file-local
        self.global_labels = [a for a in self.sorted_labels
                              if not self.labels[a][0].startswith('.L')]

    def sym(self, addr, kind, ins=None):
        """Name for an operand's address, or None to leave the number."""
        d = self.d
        if kind == 'imm':
            # a long immediate is an address where the instruction uses
            # it as one (Disasm.imm_pointer)
            if ins is None or not d.imm_pointer(ins, addr):
                return None
            if addr < HEADER_END:
                return 'header+0x%X' % (addr - 0x100)
            if addr in self.labels:
                return self.labels[addr][0]
            return self.near(addr)
        if kind == 'disp':
            if not d.disp_pointer(addr):
                return None
            return self.labels[addr][0] if addr in self.labels else \
                self.near(addr)
        if kind == 'abs' and addr in d.biased:
            t = d.biased[addr]
            return '%s-0x%X' % (self.labels[t][0], t - addr)
        if not d.in_rom(addr):
            if kind in ('pc', 'branch') and addr < HEADER_END:
                return self.near(addr)
            return None
        if addr in self.labels:
            return self.labels[addr][0]
        return self.near(addr)

    def plan_pads(self, cuts):
        """Where shift_pad goes (a SHIFT build): before each function
        that nothing falls through into (the instruction before is a
        jump or a return, and it isn't a jump table entry), as long as
        every displacement still fits.  None under an 8-bit one (`bra.s`,
        `(d8,pc,xn)`); a 16-bit one (`bra.w`, `(d16,pc)`, and a ROM
        address written `(xxx).w`, which must stay below 0x8000) takes
        as many as its slack allows, first come first served."""
        d = self.d
        cands = []
        for a in self.sorted_labels:
            if self.labels[a][1] != 'func' or a in cuts or \
                    a in d.jtbl_entries or a not in d.insns:
                continue
            p = d.owner.get(a - 2)
            if p is not None and d.insns[p].flow in ('jump', 'ret') and \
                    p + d.insns[p].size == a:
                cands.append(a)
        spans = []               # [lo, hi, slack]
        for b, ins in d.insns.items():
            for t in d.disp_refs(ins):       # a 16-bit displacement
                spans.append([0, t, 0x7FFF - t])
            for t, kind in ins.refs():
                if kind == 'absw' and d.in_rom(t):
                    spans.append([0, t, 0x7FFF - t])
                    continue
                if kind not in ('pcdisp', 'pcindex', 'label') or \
                        not d.in_rom(t):
                    continue
                if kind == 'label' and ins.flow is None:
                    continue     # not a displacement
                if kind == 'pcindex' or ins.mn.endswith('.s'):
                    slack = 0
                else:
                    base = b + 2
                    disp = t - base
                    # (base: near enough; 8 bytes to spare)
                    slack = (0x7FFF - disp if disp >= 0 else
                             0x8000 + disp) - 8
                spans.append([min(b, t) + 1, max(b, t), slack])
        for lo, hi in READS_PAST:
            spans.append([lo, hi, 0])
        spans.sort()
        import bisect
        los = [sp[0] for sp in spans]
        pads = set()
        for a in cands:
            # the spans with lo <= a <= hi
            over = [sp for sp in spans[:bisect.bisect_right(los, a)]
                    if sp[1] >= a]
            if all(sp[2] >= SHIFT_FUNC_PAD for sp in over):
                for sp in over:
                    sp[2] -= SHIFT_FUNC_PAD
                pads.add(a)
        return pads

    def ptr_name(self, t):
        if t < HEADER_END:
            return 'header+0x%X' % (t - 0x100)
        if t in self.labels:
            return self.labels[t][0]
        return self.near(t)

    def near(self, addr):
        """label+off for an address without its own label."""
        import bisect
        i = bisect.bisect_right(self.global_labels, addr) - 1
        if i < 0:
            return 'rom_start+0x%X' % addr
        base = self.global_labels[i]
        return '%s+0x%X' % (self.labels[base][0], addr - base)

    def files(self):
        """Cut [HEADER_END, end) into files: a file starts where a data
        run of DATA_SPLIT bytes or more starts, or where code starts
        after one.  Cuts are even."""
        d = self.d
        end = len(self.rom)
        # no cut inside a pc-relative reference's span: an 8-bit
        # displacement can't cross objects, and .L labels are file-local
        crossed = bytearray(end + 1)
        for a, ins in d.insns.items():
            for t, kind in ins.refs():
                if kind in ('pcdisp', 'pcindex', 'label') and d.in_rom(t):
                    lo, hi = min(a, t), max(a, t)
                    for c in range(lo + 1, hi + 1):
                        crossed[c] = 1
        # nor where a script ends: its pointer is left one past its end
        # (0xAD5E after the credits) and has to stay the next label's
        # address in a SHIFT build, with no object pad in between
        script_ends = set(a + 1 for a in d.script_bytes
                          if a + 1 not in d.script_bytes)
        cuts = [HEADER_END]
        a = HEADER_END
        in_data_file = False
        while a < end:
            if a in d.owner:
                if in_data_file and a in script_ends:
                    in_data_file = False
                if in_data_file and a & 1 == 0 and not crossed[a]:
                    cuts.append(a)
                    in_data_file = False
                a = d.owner[a] + d.insns[d.owner[a]].size
                continue
            b = a
            while b < end and b not in d.owner:
                b += 1
            if b - a >= DATA_SPLIT and not in_data_file:
                c = (a + 1) & ~1
                if c != cuts[-1] and not crossed[c]:
                    cuts.append(c)
                in_data_file = True
            a = b
        return cuts + [end]

    def write(self, outdir):
        os.makedirs(outdir, exist_ok=True)
        cuts = self.files()
        self.pads = self.plan_pads(cuts)
        # the zeros between the code and the data aren't written: the
        # data is placed at its address (`. = 0x40000` in link.ld), so a
        # SHIFT build that pads the code doesn't move it
        fixed = {}
        for lo, hi in self.d.gaps:
            lo = (lo + 1) & ~1
            if hi >= len(self.rom) or any(self.labels_in(lo, hi)):
                continue
            cuts = sorted(set([c for c in cuts if not lo < c < hi] +
                              [lo, hi]))
            fixed[lo] = hi
        objs = ['header']
        lines = ['BUILD_DIR/asm/header.o(.text)']
        self.write_header(os.path.join(outdir, 'header.s'))
        for lo, hi in zip(cuts, cuts[1:]):
            if lo in fixed:
                lines.append('. = 0x%06X;' % hi)
                continue
            name = self.d.tag(lo)    # us's address in every version
            objs.append(name)
            lines.append('BUILD_DIR/asm/%s.o(.text)' % name)
            self.write_file(os.path.join(outdir, name + '.s'), lo, hi)
        with open(os.path.join(outdir, 'link.ld'), 'w') as f:
            f.write('/* generated by tools/disasm.py */\n')
            for line in lines:
                f.write(line + '\n')
        with open(os.path.join(outdir, 'symbols.txt'), 'w') as f:
            for a in self.sorted_labels:
                name, kind = self.labels[a]
                if not name.startswith('.L'):
                    f.write('%06X %s %s\n' % (a, name, kind))
        return objs

    def prologue(self, f):
        f.write('| generated by tools/disasm.py; do not edit\n')
        f.write('\t.include "macros.inc"\n\n')

    def write_header(self, path):
        rom = self.rom
        with open(path, 'w') as f:
            self.prologue(f)
            f.write('vectors:\n')
            vec = struct.unpack('>64I', rom[:256])
            for i, v in enumerate(vec):
                name = self.sym(v, 'abs') if v >= HEADER_END and \
                    v < len(rom) else None
                f.write('\t.long %s\t| %d\n' % (name or '0x%08X' % v, i))
            f.write('\t.global header\nheader:\n')
            self.data(f, 0x100, 0x200, text=True)

    def data(self, f, lo, hi, text=False):
        a = lo
        rom = self.rom
        while a < hi:
            if a in self.d.ptrs and a + 4 <= hi:
                f.write('\t.long %s\n' % self.ptr_name(self.d.ptrs[a]))
                a += 4
                if a in self.labels and a < hi:
                    self.label(f, a)
                continue
            # stop each run at the next label or pointer
            nxt = hi
            for la in self.labels_in(a + 1, hi):
                nxt = la
                break
            nxt = min([nxt] + [p for p in range(a + 1, min(nxt, a + 32))
                               if p in self.d.ptrs])
            z = zero_run(rom, a, nxt)
            if z >= ZERO_RUN:
                f.write('\t.space 0x%X\n' % z)
                a += z
            else:
                n = min(16, nxt - a)
                # don't run into a zero run worth a .space
                for k in range(n):
                    if zero_run(rom, a + k, nxt) >= ZERO_RUN:
                        n = k
                        break
                n = max(n, 1)
                f.write('\t.byte %s\n' % ','.join(
                    '0x%02X' % b for b in rom[a:a + n]))
                a += n
            if a in self.labels and a < hi:
                self.label(f, a)

    def labels_in(self, lo, hi):
        import bisect
        i = bisect.bisect_left(self.sorted_labels, lo)
        while i < len(self.sorted_labels) and self.sorted_labels[i] < hi:
            yield self.sorted_labels[i]
            i += 1

    def label(self, f, a):
        name, kind = self.labels[a]
        if kind == 'func':
            f.write('\n')
        if not name.startswith('.L'):
            f.write('\t.global %s\n' % name)
        f.write('%s:\n' % name)

    def write_file(self, path, lo, hi):
        d = self.d
        rom = self.rom
        with open(path, 'w') as f:
            self.prologue(f)
            a = lo
            prev = None          # the instruction before a, if any
            while a < hi:
                if a in self.labels:
                    if a in self.pads:
                        f.write('\n\tshift_pad')
                    self.label(f, a)
                prev = d.insns.get(a)
                if a in d.insns:
                    ins = d.insns[a]
                    if ins.raw:
                        words = struct.unpack('>%dH' % (ins.size // 2),
                                              rom[a:a + ins.size])
                        f.write('\t.short %s\t| %s\n' % (
                            ','.join('0x%04X' % w for w in words),
                            ins.format(self.sym)))
                    else:
                        f.write('\t%s\n' % ins.format(
                            lambda t, k: self.sym(t, k, ins)))
                    a += ins.size
                    continue
                b = a
                while b < hi and b not in d.insns:
                    b += 1
                self.data(f, a, b)
                a = b


def load_known(version):
    """code.VERSION.txt (`ADDR`) and data.VERSION.txt (`LO HI`, hi
    exclusive): what runtime traces proved (tools/coverage.py --write)."""
    def lines(name):
        path = os.path.join(TOP, '%s.%s.txt' % (name, version))
        if not os.path.exists(path):
            return []
        out = []
        for line in open(path):
            t = line.split('#')[0].split()
            if t:
                out.append([int(x, 16) for x in t])
        return out
    return ([t[0] for t in lines('code')],
            [(t[0], t[1]) for t in lines('data')],
            [t[0] for t in lines('longs')])


VERSIONS = ('us', 'eu')


def analyze(version, speculate=True, versions=True):
    """The whole analysis of baserom.VERSION.bin, ready to write.  With
    versions, the other version is analysed too and compared through the
    version map (tools/vermap.py): outside us, names come from us
    (Disasm.tag), and immediates that stay put where their target moves
    are numbers."""
    def core(v):
        rom = open(os.path.join(TOP, 'baserom.%s.bin' % v), 'rb').read()
        extra, data, longs = load_known(v)
        d = Disasm(rom, extra, data, longs)
        d.version = v
        d.biased = dict(BIASED.get(v, {}))
        d.scripts = SCRIPTS.get(v, [])
        d.records = RECORDS.get(v, [])
        d.trace_all()
        if speculate:
            d.speculate_all()
        return d
    d = core(version)
    if versions and speculate:
        import vermap
        other = [v for v in VERSIONS if v != version][0]
        od = core(other)
        ref, oth = (d, od) if version == vermap.REF else (od, d)
        vm = vermap.VerMap(vermap.build(oth.version, ref, oth))
        vermap.numbers(ref, oth, vm)
        vermap.moved_longs(ref, oth, vm)
        if d is oth:
            d.vmap = vm
    d.collect_refs()
    return d


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('version')
    ap.add_argument('--out', default=os.path.join(TOP, 'asm'))
    ap.add_argument('--stats', action='store_true')
    ap.add_argument('--no-speculate', action='store_true',
                    help='only what tracing from the vectors finds')
    a = ap.parse_args()
    d = analyze(a.version, not a.no_speculate)
    rom = d.rom
    w = Writer(d, rom, a.version)
    objs = w.write(a.out)
    with open(os.path.join(TOP, '.version'), 'w') as f:
        f.write(a.version + '\n')
    code = sum(i.size for i in d.insns.values())
    print('%s: %d instructions, %d bytes of code (%.1f%% of 0x%X), '
          '%d functions, %d files, %d conflicts' % (
              a.version, len(d.insns), code, 100.0 * code / len(rom),
              len(rom), len([f for f in d.funcs if f in d.insns]),
              len(objs), len(d.conflicts)))
    print('    %d roots by speculation, code area below 0x%X' % (
        len(d.probable), d.code_limit))
    print('    %d pointer longs in data (%d read by a trace, %d in scripts)'
          % (len(d.ptrs), len(d.longs & set(d.ptrs)), len(d.script_ptrs)))
    if a.stats:
        for c in sorted(set(d.conflicts))[:40]:
            print('  conflict at %06X' % c)


if __name__ == '__main__':
    main()
