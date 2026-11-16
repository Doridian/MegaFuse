#!/usr/bin/env python3
"""The Z80 sound driver translated into Rust (docs/PORT.md, "Sound").

    tools/zrecomp.py [--out crates/md/src/driver.rs]

The driver the game uploads (tools/z80.py, us's at 0xF8000) is cut into
routines as tools/recomp cuts the 68000's: an entry and what it reaches
without passing another, where the entries are 0x0000, the targets of
calls and `rst`, and the places `jp (hl)` went (zcode.us.txt, the
traces').  Each routine is structured the same way (recomp/structure.py)
and becomes a function on `Sound` that keeps the Z80's registers in its
`Z80` and goes through the same bus, instruction by instruction at the
interpreter's time (`Sound::zin`, `Sound::zout`), with the interpreter's
arithmetic (z80.rs).  A call is a call: the return address is pushed to
the Z80's stack as the Z80 would, and the callee's `ret` hands back the
address it popped.

Where the translation can't go on, it leaves the Z80 to the interpreter
at an instruction (`Flow::Elsewhere`, the PC set): out of time (the
68000 wants the bus, a segment ends), `jp (hl)`, an instruction it
doesn't translate (`ei`, `halt`, I/O), a return somewhere else than
after its call.  The interpreter then runs until the PC is at an entry
or a place to resume: each of a routine's basic blocks is a function
too, which the runner goes on from to the next (so that a routine left
in the middle, its callers' Rust frames gone, comes back to the
translation at the end of the block it was in).
"""
import argparse
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TOP = os.path.dirname(HERE)
sys.path.insert(0, HERE)

import z80                          # noqa: E402
from recomp import structure        # noqa: E402

R8 = ('a', 'b', 'c', 'd', 'e', 'h', 'l')
R16 = ('bc', 'de', 'hl', 'sp', 'ix', 'iy', 'af')
CC = {c: k for k, c in enumerate(z80.CC)}
ALU = {'add': 0, 'adc': 1, 'sub': 2, 'sbc': 3, 'and': 4, 'xor': 5,
       'or': 6, 'cp': 7}
ROT = {m: k for k, m in enumerate(z80.ROT)}
# what each conditional instruction takes when it goes (the interpreter's)
TAKEN = {'jr': 12, 'djnz': 13, 'ret': 11, 'call': 17}


def num(s):
    return int(s, 0)


def is_num(s):
    return re.fullmatch(r'0x[0-9A-Fa-f]+|\d+', s) is not None


def split_ops(text):
    mn, _, rest = text.partition(' ')
    return mn, [o for o in rest.split(',')] if rest else []


# --- time ------------------------------------------------------------------

def base_cycles(ins):
    """What the instruction takes but for a branch taken or a block
    instruction going round (Z80::base_cycles, the interpreter's)."""
    mn, ops = split_ops(ins.text)
    page = ins.op
    idx = page in (0xDD, 0xFD)
    mem = any(o.startswith('(i') and o[3:4] in '+-' for o in ops)
    hl_mem = '(hl)' in ops
    if page == 0xED:
        if mn in ('in', 'out'):
            return 12
        if mn in ('adc', 'sbc'):
            return 15
        if mn == 'ld':
            if ops[0] in ('i', 'r') or ops[1] in ('i', 'r'):
                return 9
            return 20
        if mn == 'neg':
            return 8
        if mn in ('reti', 'retn'):
            return 14
        if mn == 'im':
            return 8
        if mn in ('rrd', 'rld'):
            return 18
        if mn in ('ldi', 'ldd', 'cpi', 'cpd', 'ldir', 'lddr', 'cpir',
                  'cpdr', 'ini', 'ind', 'outi', 'outd', 'inir', 'indr',
                  'otir', 'otdr'):
            return 16
        return 8
    if ins.size >= 2 and (ins.text.split(' ')[0] in ROT or mn in
                          ('bit', 'res', 'set')) and \
            (page == 0xCB or (idx and mem)):
        if idx:
            return (20 if mn == 'bit' else 23)
        if hl_mem:
            return 12 if mn == 'bit' else 15
        return 8
    extra = 4 if idx else 0
    if mn == 'ld':
        d, s = ops
        if d in R16 or s in R16:
            if d == 'sp' and s in ('hl', 'ix', 'iy'):
                return 6 + extra
            if is_num(s):
                return 10 + extra
            return 16 + extra          # ld rr,(nn) / ld (nn),rr (main page)
        if d in ('(bc)', '(de)') or s in ('(bc)', '(de)'):
            return 7
        if re.fullmatch(r'\((0x[0-9A-F]+|\d+)\)', d) or \
                re.fullmatch(r'\((0x[0-9A-F]+|\d+)\)', s):
            return 13
        if mem:
            return 15 + extra
        if hl_mem:
            return 10 if is_num(s) else 7
        if is_num(s):
            return 7 + extra
        return 4 + extra
    if mn in ('inc', 'dec'):
        o = ops[0]
        if o in R16:
            return 6 + extra
        if mem:
            return 19 + extra
        if hl_mem:
            return 11
        return 4 + extra
    if mn in ALU:
        o = ops[-1]
        if mn in ('add', 'adc', 'sbc') and ops[0] in ('hl', 'ix', 'iy'):
            return 11 + extra
        if mem:
            return 15 + extra
        if hl_mem or is_num(o):
            return 7
        return 4 + extra
    if mn in ('nop', 'rlca', 'rrca', 'rla', 'rra', 'daa', 'cpl', 'scf',
              'ccf', 'exx', 'di', 'ei', 'halt'):
        return 4
    if mn == 'ex':
        if ops[0] == '(sp)':
            return 19 + extra
        return 4
    if mn == 'djnz':
        return 8
    if mn == 'jr':
        return 12 if len(ops) == 1 else 7
    if mn == 'jp':
        if ops[0].startswith('('):
            return 4 + extra
        return 10
    if mn == 'ret':
        return 10 if not ops else 5
    if mn == 'pop':
        return 10 + extra
    if mn == 'push':
        return 11 + extra
    if mn == 'call':
        return 17 if len(ops) == 1 else 10
    if mn == 'rst':
        return 11
    if mn in ('out', 'in'):
        return 11
    raise ValueError('no time for %r' % ins.text)


# --- operands --------------------------------------------------------------

def mem_addr(o):
    """The address expression of a memory operand, or None."""
    m = re.fullmatch(r'\((i[xy])([+-]\d+)\)', o)
    if m:
        d = int(m.group(2)) & 0xFFFF
        return 'self.z80.%s.wrapping_add(0x%04X)' % (m.group(1), d)
    if o in ('(hl)', '(bc)', '(de)'):
        return 'self.z80.%s()' % o[1:3]
    m = re.fullmatch(r'\((0x[0-9A-Fa-f]+|\d+)\)', o)
    if m:
        return '0x%04X_u16' % num(m.group(1))
    return None


def rd8(o):
    if o in R8:
        return 'self.z80.%s' % o
    if o in ('ixh', 'iyh'):
        return '(self.z80.%s >> 8) as u8' % o[:2]
    if o in ('ixl', 'iyl'):
        return 'self.z80.%s as u8' % o[:2]
    a = mem_addr(o)
    if a:
        return 'self.bus.read(%s)' % a
    if is_num(o):
        return '0x%02X_u8' % num(o)
    raise ValueError(o)


def wr8(o, v):
    if o in R8:
        return ['self.z80.%s = %s;' % (o, v)]
    if o in ('ixh', 'iyh'):
        r = o[:2]
        return ['self.z80.%s = (self.z80.%s & 0x00FF) | (u16::from(%s) << 8);'
                % (r, r, v)]
    if o in ('ixl', 'iyl'):
        r = o[:2]
        return ['self.z80.%s = (self.z80.%s & 0xFF00) | u16::from(%s);'
                % (r, r, v)]
    a = mem_addr(o)
    if a:
        return ['self.bus.write(%s, %s);' % (a, v)]
    raise ValueError(o)


def rd16(r):
    if r in ('bc', 'de', 'hl', 'af'):
        return 'self.z80.%s()' % r
    return 'self.z80.%s' % r


def wr16(r, v):
    if r in ('bc', 'de', 'hl', 'af'):
        return ['self.z80.set_%s(%s);' % (r, v)]
    return ['self.z80.%s = %s;' % (r, v)]


# --- instructions ----------------------------------------------------------

class Fallback(Exception):
    """Left to the interpreter."""


def body(ins):
    """The Rust for an instruction that falls through (not a branch,
    jump, call or return): lines between zin and zout."""
    mn, ops = split_ops(ins.text)
    if ins.text.startswith('nop'):
        return []
    if mn == 'ld':
        d, s = ops
        if d in ('i', 'r') or s in ('i', 'r'):
            raise Fallback
        if d in R16 or s in R16:
            if d in R16 and is_num(s):
                return wr16(d, '0x%04X' % num(s))
            if d in R16 and s in R16:
                return wr16(d, rd16(s))
            if d in R16:
                return ['let v = Z80::read16(&mut self.bus, %s);' %
                        mem_addr(s)] + wr16(d, 'v')
            return ['let v = %s;' % rd16(s),
                    'Z80::write16(&mut self.bus, %s, v);' % mem_addr(d)]
        return ['let v = %s;' % rd8(s)] + wr8(d, 'v')
    if mn == 'push':
        return ['let v = %s;' % rd16(ops[0]),
                'self.z80.push(&mut self.bus, v);']
    if mn == 'pop':
        return ['let v = self.z80.pop(&mut self.bus);'] + wr16(ops[0], 'v')
    if mn == 'ex':
        if ops == ['de', 'hl']:
            return ['self.z80.ex_de_hl();']
        if ops[0] == 'af':
            return ['self.z80.ex_af();']
        r = ops[1]
        return ['let v = Z80::read16(&mut self.bus, self.z80.sp);',
                'let w = %s;' % rd16(r),
                'Z80::write16(&mut self.bus, self.z80.sp, w);'] + wr16(r, 'v')
    if mn == 'exx':
        return ['self.z80.exx();']
    if mn in ALU:
        if mn in ('add', 'adc', 'sbc') and ops[0] in ('hl', 'ix', 'iy'):
            f = {'add': 'add16', 'adc': 'adc16', 'sbc': 'sbc16'}[mn]
            return ['let r = self.z80.%s(%s, %s);' % (f, rd16(ops[0]),
                                                       rd16(ops[1]))] + \
                wr16(ops[0], 'r')
        return ['let v = %s;' % rd8(ops[-1]),
                'self.z80.alu(%d, v);' % ALU[mn]]
    if mn in ('inc', 'dec'):
        o = ops[0]
        if o in R16:
            f = 'wrapping_add' if mn == 'inc' else 'wrapping_sub'
            return wr16(o, '%s.%s(1)' % (rd16(o), f))
        return ['let v = %s;' % rd8(o),
                'let r = self.z80.%s8(v);' % mn] + wr8(o, 'r')
    if mn in ('rlca', 'rrca', 'rla', 'rra'):
        return ['self.z80.rot_a(%d);' % ['rlca', 'rrca', 'rla',
                                         'rra'].index(mn)]
    if mn in ('daa', 'cpl', 'scf', 'ccf', 'neg', 'di'):
        return ['self.z80.%s();' % mn]
    if mn == 'im':
        return ['self.z80.set_im(%d);' % num(ops[0])]
    if mn in ROT:
        o = ops[0]
        out = ['let v = %s;' % rd8(o), 'let r = self.z80.rot(%d, v);' %
               ROT[mn]] + wr8(o, 'r')
        if len(ops) == 2:
            out += wr8(ops[1], 'r')
        return out
    if mn == 'bit':
        return ['let v = %s;' % rd8(ops[1]),
                'self.z80.bit(%d, v);' % num(ops[0])]
    if mn in ('res', 'set'):
        y = num(ops[0])
        o = ops[1]
        op = '& !0x%02X' % (1 << y) if mn == 'res' else '| 0x%02X' % (1 << y)
        out = ['let r = %s %s;' % (rd8(o), op)] + wr8(o, 'r')
        if len(ops) == 3:
            out += wr8(ops[2], 'r')
        return out
    if mn in ('ldi', 'ldd'):
        return ['self.z80.ld_block(&mut self.bus, %s);' %
                ('true' if mn == 'ldd' else 'false')]
    if mn in ('cpi', 'cpd'):
        return ['self.z80.cp_block(&mut self.bus, %s);' %
                ('true' if mn == 'cpd' else 'false')]
    raise Fallback


REPEAT = {'ldir': ('ld_block', 'false'), 'lddr': ('ld_block', 'true'),
          'cpir': ('cp_block', 'false'), 'cpdr': ('cp_block', 'true')}


# --- routines --------------------------------------------------------------

class ZInsn:
    """tools/z80.py's instruction as recomp/structure.py reads one."""
    __slots__ = ('z', 'addr', 'size', 'flow', 'target', 'text')

    def __init__(self, z):
        self.z = z
        self.addr = z.addr
        self.size = z.size
        self.text = z.text
        self.target = z.target
        self.flow = {None: None, 'call': 'call', 'ccall': 'call',
                     'cret': None, 'branch': 'branch', 'jump': 'jump',
                     'ret': 'ret', 'jumpind': 'jump',
                     'halt': 'stop'}[z.flow]
        if z.flow == 'jumpind':
            self.target = None


class Code:
    def __init__(self, insns):
        self.insns = insns


def find_code(mem, ran):
    """Every instruction: by flow from 0x0000 and from what ran."""
    return z80.walk(mem, [0] + sorted(ran))


def find_entries(code, ran):
    entries = {0}
    for i in code.values():
        if i.flow in ('call', 'ccall') and i.target in code:
            entries.add(i.target)
    # what ran that nothing reaches but through `jp (hl)`: a table's
    entries |= set(a for a in jump_targets(code, entries, ran))
    return entries


def succ(code, a):
    i = code[a]
    out = []
    if i.flow in (None, 'call', 'ccall', 'cret', 'branch'):
        out.append(a + i.size)
    if i.flow in ('branch', 'jump') and i.target is not None:
        out.append(i.target)
    return [t for t in out if t in code]


def jump_targets(code, entries, ran):
    out = set()
    while True:
        seen = set()
        todo = list(entries | out)
        while todo:
            a = todo.pop()
            if a in seen or a not in code:
                continue
            seen.add(a)
            todo += succ(code, a)
            i = code[a]
            if i.flow in ('call', 'ccall') and i.target in code:
                todo.append(i.target)
        rest = sorted(a for a in ran if a in code and a not in seen)
        if not rest:
            return out
        out.add(rest[0])


def partition(code, entries):
    """routine entry -> its instructions; an instruction two routines
    reach becomes an entry of its own."""
    entries = set(entries)
    while True:
        owner = {}
        shared = set()
        bodies = {}
        for e in sorted(entries):
            body_ = []
            seen = {e}
            todo = [e]
            while todo:
                a = todo.pop()
                body_.append(a)
                if a in owner and owner[a] != e:
                    shared.add(a)
                owner.setdefault(a, e)
                for t in succ(code, a):
                    if t not in seen and t not in entries:
                        seen.add(t)
                        todo.append(t)
            # the entry first (recomp/structure.py starts there)
            bodies[e] = [e] + sorted(x for x in body_ if x != e)
        if not shared:
            return entries, bodies
        # the first shared instruction of each conflict: an entry
        for a in sorted(shared):
            preds = [p for p in code if a in succ(code, p)]
            if any(p not in shared for p in preds) or not preds:
                entries.add(a)


def resume_points(code, e, body_):
    """Where a routine is taken up again: its basic blocks (a branch's
    target, what follows a branch, a call or a conditional return)."""
    inside = set(body_)
    out = set()
    for a in body_:
        i = code[a]
        if i.flow in ('branch', 'jump') and i.target in inside:
            out.add(i.target)
        if i.flow in ('branch', 'call', 'ccall', 'cret'):
            out.add(a + i.size)
    return sorted(r for r in out if r in inside and r != e)


def region(code, body_, r, points):
    """The routine from r on, as far as the next place to resume (the
    runner goes on there): what r reaches within it without passing one
    of points."""
    inside = set(body_)
    seen = {r}
    todo = [r]
    while todo:
        a = todo.pop()
        for t in succ(code, a):
            if t in inside and t not in seen and t not in points:
                seen.add(t)
                todo.append(t)
    return [r] + sorted(x for x in seen if x != r)


def fname(a):
    return 'z_%04x' % a


class Emitter:
    def __init__(self, code, entries):
        self.code = code
        self.entries = entries
        self.fallbacks = 0

    def zi(self, a, base):
        return ['zi!(self, 0x%04X, %d);' % (a, base)]

    def leave(self, a):
        """To the interpreter at a."""
        return ['self.z80.pc = 0x%04X;' % a, 'return Flow::Elsewhere;']

    def tail(self, t):
        if t in self.entries:
            return ['return self.%s();' % fname(t)]
        return self.leave(t)

    def call(self, t, nxt):
        """After the call's push: the callee, then on after it if it
        came back there."""
        if t not in self.entries:
            return self.leave(t)
        return ['match self.%s() {' % fname(t),
                '    Flow::Ret(0x%04X) => {}' % nxt,
                '    Flow::Ret(a) => {',
                '        self.z80.pc = a;',
                '        return Flow::Elsewhere;',
                '    }',
                '    Flow::Elsewhere => return Flow::Elsewhere,',
                '}']

    def insn(self, a):
        """An instruction that doesn't end its block (or a call)."""
        i = self.code[a]
        mn, ops = split_ops(i.text)
        nxt = a + i.size
        comment = ['// %04X  %s' % (a, i.text)]
        if mn in REPEAT:
            f, down = REPEAT[mn]
            return comment + [
                'loop {',
                '    zi!(self, 0x%04X, 16);' % a,
                '    if self.z80.%s(&mut self.bus, %s) {' % (f, down),
                '        self.zout(21);',
                '    } else {',
                '        self.zout(16);',
                '        break;',
                '    }',
                '}']
        try:
            if mn in ('call', 'rst'):
                base = base_cycles(i)
                t = i.target
                if mn == 'call' and len(ops) == 2:
                    return comment + self.zi(a, base) + [
                        'if self.z80.cond(%d) {' % CC[ops[0]],
                        '    self.z80.push(&mut self.bus, 0x%04X);' % nxt,
                        '    self.zout(17);'] + \
                        ['    ' + x for x in self.call(t, nxt)] + [
                        '} else {', '    self.zout(%d);' % base, '}']
                return comment + self.zi(a, base) + [
                    'self.z80.push(&mut self.bus, 0x%04X);' % nxt,
                    'self.zout(%d);' % base] + self.call(t, nxt)
            if mn == 'ret' and ops:
                return comment + self.zi(a, 5) + [
                    'if self.z80.cond(%d) {' % CC[ops[0]],
                    '    let a = self.z80.pop(&mut self.bus);',
                    '    self.zout(11);',
                    '    return Flow::Ret(a);',
                    '}',
                    'self.zout(5);']
            lines = body(i)
        except Fallback:
            self.fallbacks += 1
            return comment + self.leave(a)
        base = base_cycles(i)
        return comment + self.zi(a, base) + lines + ['self.zout(%d);' % base]

    def last(self, b):
        """A block's last instruction, where it is a transfer."""
        i = self.code[b.insns[-1]]
        return i

    def node(self, n):
        k = n[0]
        if k == 'seq':
            out = []
            for x in n[1]:
                out += self.node(x)
            return out
        if k == 'code':
            b = self.blocks[n[1]]
            out = []
            for a in b.insns[:-1]:
                out += self.insn(a)
            i = self.code[b.insns[-1]]
            if i.flow in ('branch', 'ret', 'jumpind', 'halt') or \
                    (i.flow == 'jump' and b.kind != 'jump'):
                return out                 # the 'if' or 'exit' has it
            if i.flow == 'jump':
                # an internal jump: its time, then the branch node
                base = base_cycles(i)
                return out + ['// %04X  %s' % (i.addr, i.text)] + \
                    self.zi(i.addr, base) + ['self.zout(%d);' % base]
            return out + self.insn(i.addr)
        if k == 'if':
            b = self.blocks[n[1]]
            i = self.code[b.insns[-1]]
            mn, ops = split_ops(i.text)
            base = base_cycles(i)
            out = ['// %04X  %s' % (i.addr, i.text)] + self.zi(i.addr, base)
            if mn == 'djnz':
                out += ['self.z80.b = self.z80.b.wrapping_sub(1);',
                        'let t = self.z80.b != 0;']
            else:
                out += ['let t = self.z80.cond(%d);' % CC[ops[0]]]
            taken = TAKEN.get(mn, base)
            if taken == base:
                out += ['self.zout(%d);' % base]
            else:
                out += ['self.zout(if t { %d } else { %d });' % (taken, base)]
            out += ['if t {'] + ind(self.node(n[2])) + ['} else {'] + \
                ind(self.node(n[3])) + ['}']
            return out
        if k == 'labeled':
            return ['%s: {' % n[1]] + ind(self.node(n[2])) + ['}']
        if k == 'loop':
            return ['%s: loop {' % n[1]] + ind(self.node(n[2])) + ['}']
        if k == 'break':
            return ['break %s;' % n[1]]
        if k == 'continue':
            return ['continue %s;' % n[1]]
        if k == 'tail':
            return self.tail(n[1])
        if k == 'exit':
            b = self.blocks[n[1]]
            i = self.code[b.insns[-1]]
            mn, ops = split_ops(i.text)
            head = ['// %04X  %s' % (i.addr, i.text)]
            if i.flow == 'ret':
                if mn != 'ret':           # reti, retn
                    self.fallbacks += 1
                    return head + self.leave(i.addr)
                return head + self.zi(i.addr, 10) + [
                    'let a = self.z80.pop(&mut self.bus);',
                    'self.zout(10);', 'return Flow::Ret(a);']
            if i.flow == 'jump':
                base = base_cycles(i)
                return head + self.zi(i.addr, base) + \
                    ['self.zout(%d);' % base] + self.tail(i.target)
            if i.flow == 'jumpind':
                base = base_cycles(i)
                r = ops[0][1:-1]
                return head + self.zi(i.addr, base) + [
                    'self.z80.pc = %s;' % rd16(r),
                    'self.zout(%d);' % base, 'return Flow::Elsewhere;']
            if i.flow == 'halt':
                self.fallbacks += 1
                return head + self.leave(i.addr)
            # falls into another routine
            return self.tail(i.addr + i.size)
        raise ValueError(n)

    def routine(self, e, body_, of=None):
        d = Code({a: ZInsn(self.code[a]) for a in body_})
        self.blocks, tree = structure.structure(d, body_)
        lines = self.node(tree)
        doc = '/// %04X' % e if of is None else \
            '/// %04X, from %04X on (resumed there)' % (of, e)
        return [doc,
                '#[allow(unreachable_code, unused_labels, '
                'clippy::all)]',
                'pub(crate) fn %s(&mut self) -> Flow {' % fname(e)] + \
            ind(lines) + ['}']


def ind(lines):
    return ['    ' + x for x in lines]


def load_ran(version='us'):
    path = os.path.join(TOP, 'zcode.%s.txt' % version)
    out = set()
    for line in open(path):
        line = line.split('#')[0].strip()
        if line:
            out.add(int(line, 16))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('--rom', default=os.path.join(TOP, 'baserom.us.bin'))
    ap.add_argument('--out', default=os.path.join(TOP, 'crates', 'md', 'src',
                                                  'driver.rs'))
    a = ap.parse_args()
    mem = z80.driver(open(a.rom, 'rb').read())
    ran = load_ran()
    code = find_code(mem, ran)
    entries = find_entries(code, ran)
    entries, bodies = partition(code, entries)
    em = Emitter(code, entries)
    out = []
    funcs = []
    for e in sorted(bodies):
        out += em.routine(e, bodies[e]) + ['']
        funcs.append(e)
        points = resume_points(code, e, bodies[e])
        # a part ends where another starts, and at the routine's entry
        ends = set(points) | {e}
        for r in points:
            try:
                out += em.routine(r, region(code, bodies[e], r, ends),
                                  e) + ['']
            except structure.Irreducible:
                continue
            funcs.append(r)
    with open(a.out, 'w') as f:
        f.write('// generated by tools/zrecomp.py; do not edit\n')
        f.write('//! The Z80 sound driver, translated (tools/zrecomp.py, '
                'docs/PORT.md "Sound").\n\n')
        f.write('use crate::sound::{Flow, Sound};\nuse crate::z80::{Bus, Z80};\n\n')
        f.write('/// The driver as uploaded (us), whose code this is: '
                'its length and a hash.\n')
        f.write('pub(crate) const DRIVER: (usize, u64) = (0x%X, 0x%016X);\n\n'
                % (len(mem), fnv(mem)))
        f.write('macro_rules! zi {\n    ($s:ident, $pc:expr, $base:expr) => '
                '{\n        if !$s.zin($pc, $base) {\n            '
                'return Flow::Elsewhere;\n        }\n    };\n}\n\n')
        f.write('impl Sound {\n')
        f.write('    /// The routine at `pc`, if there is one.\n')
        f.write('    pub(crate) fn driver_at(&mut self, pc: u16) -> '
                'Option<Flow> {\n        Some(match pc {\n')
        for e in sorted(funcs):
            f.write('            0x%04X => self.%s(),\n' % (e, fname(e)))
        f.write('            _ => return None,\n        })\n    }\n\n')
        for line in out:
            f.write(('    ' + line).rstrip() + '\n')
        f.write('}\n')
    print('%d instructions, %d routines, %d places to resume, %d left to '
          'the interpreter' % (len(code), len(bodies), len(funcs) -
                               len(bodies), em.fallbacks))


def fnv(b):
    h = 0xCBF29CE484222325
    for x in b:
        h = ((h ^ x) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


if __name__ == '__main__':
    main()
