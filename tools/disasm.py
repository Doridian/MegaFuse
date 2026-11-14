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
  - code.VERSION.txt: addresses known to be code by other evidence, one
    per line (`ADDR  # comment`, in VERSION's addresses).

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
  asm/link.ld         the objects in ROM order
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


def is_ram(a):
    return a >= 0xFF0000


def pointer_imm(ins):
    """Is ins's long immediate used as an address?  `movea.l #x,an`, and
    `move.l #x` into memory or onto the stack (a pushed return address);
    `move.l #x,dn` or `cmpi.l #x` may as well be a number."""
    if ins.mn == 'movea.l':
        return True
    return ins.mn == 'move.l' and ins.ops[1].kind != 'dn'


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
    def __init__(self, rom, extra=()):
        self.rom = rom
        self.end = len(rom)
        self.insns = {}          # addr -> Insn
        self.owner = {}          # byte addr -> insn start, for code bytes
        self.funcs = set()       # called or entry
        self.branch = set()      # branch targets
        self.jtbl = set()        # jump table starts
        self.dlabels = set()     # data addresses the code names
        self.conflicts = []
        self.slots = set()       # RAM longs jumped through
        self.slot_stores = collections.defaultdict(set)
        self.extra = set(extra)
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
            if not n:
                break

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

    def _speculate_named(self):
        n = 0
        for a in self.candidates():
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
                    break
            return
        if mn == 'move.l' and ins.ops[0].kind == 'imm' and \
                ins.ops[1].kind in ('absw', 'absl') and \
                is_ram(ins.ops[1].addr):
            st = self.slot_stores[ins.ops[1].addr]
            self._add(st, ins.ops[0].val)

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
            todo.append(a)
            a += 4

    # -- references --------------------------------------------------------

    def collect_refs(self):
        """Data labels: ROM addresses named by code that aren't code."""
        self.absref = set()      # named other than pc-relatively
        for a, ins in self.insns.items():
            for t, kind in ins.refs():
                if kind != 'label':
                    self.absref.add(t)
                if kind == 'imm':
                    continue
                if self.in_rom(t) and t not in self.insns and \
                        kind != 'label':
                    self.dlabels.add(t)

    def labels(self):
        """addr -> (name, kind) for every label emitted."""
        out = {}
        for a in self.dlabels:
            if a in self.owner and self.owner[a] != a:
                continue         # inside an instruction: named relative
            out[a] = ('D_%06X' % a, 'data')
        for a in self.jtbl:
            out[a] = ('jtbl_%06X' % a, 'jtbl')
        for a in self.branch:
            if a in self.insns:
                if a in self.absref or a in self.extra:
                    out[a] = ('loc_%06X' % a, 'loc')
                else:
                    out[a] = ('.L%06X' % a, 'branch')
        for a in self.funcs:
            if a in self.insns:
                out[a] = ('func_%06X' % a, 'func')
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

    def sym(self, addr, kind):
        """Name for an operand's address, or None to leave the number."""
        d = self.d
        if kind == 'imm':
            # a long immediate is an address only where it names code
            # or a data label the code also names in an address operand
            if addr in self.labels and addr >= HEADER_END:
                return self.labels[addr][0]
            return None
        if not d.in_rom(addr):
            if kind in ('pc', 'branch') and addr < HEADER_END:
                return self.near(addr)
            return None
        if addr in self.labels:
            return self.labels[addr][0]
        return self.near(addr)

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
        cuts = [HEADER_END]
        a = HEADER_END
        in_data_file = False
        while a < end:
            if a in d.owner:
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
        objs = ['header']
        self.write_header(os.path.join(outdir, 'header.s'))
        for lo, hi in zip(cuts, cuts[1:]):
            name = '%06X' % lo
            objs.append(name)
            self.write_file(os.path.join(outdir, name + '.s'), lo, hi)
        with open(os.path.join(outdir, 'link.ld'), 'w') as f:
            f.write('/* generated by tools/disasm.py */\n')
            for o in objs:
                f.write('BUILD_DIR/asm/%s.o(.text)\n' % o)
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
            f.write('header:\n')
            self.data(f, 0x100, 0x200, text=True)

    def data(self, f, lo, hi, text=False):
        a = lo
        rom = self.rom
        while a < hi:
            # stop each run at the next label
            nxt = hi
            for la in self.labels_in(a + 1, hi):
                nxt = la
                break
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
            while a < hi:
                if a in self.labels:
                    self.label(f, a)
                if a in d.insns:
                    ins = d.insns[a]
                    if ins.raw:
                        words = struct.unpack('>%dH' % (ins.size // 2),
                                              rom[a:a + ins.size])
                        f.write('\t.short %s\t| %s\n' % (
                            ','.join('0x%04X' % w for w in words),
                            ins.format(self.sym)))
                    else:
                        f.write('\t%s\n' % ins.format(self.sym))
                    a += ins.size
                    continue
                b = a
                while b < hi and b not in d.insns:
                    b += 1
                self.data(f, a, b)
                a = b


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('version')
    ap.add_argument('--out', default=os.path.join(TOP, 'asm'))
    ap.add_argument('--stats', action='store_true')
    ap.add_argument('--no-speculate', action='store_true',
                    help='only what tracing from the vectors finds')
    a = ap.parse_args()
    rom = open(os.path.join(TOP, 'baserom.%s.bin' % a.version), 'rb').read()
    extra = []
    known = os.path.join(TOP, 'code.%s.txt' % a.version)
    if os.path.exists(known):
        for line in open(known):
            line = line.split('#')[0].split()
            if line:
                extra.append(int(line[0], 16))
    d = Disasm(rom, extra)
    d.trace_all()
    if not a.no_speculate:
        d.speculate_all()
    d.collect_refs()
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
    if a.stats:
        for c in sorted(set(d.conflicts))[:40]:
            print('  conflict at %06X' % c)


if __name__ == '__main__':
    main()
