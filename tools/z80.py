#!/usr/bin/env python3
"""A Z80 decoder, for the sound driver the game uploads (0xF8000 in us).

    tools/z80.py [ROM] [--at ADDR]   # the driver, from its entry, by flow

decode(mem, pc) -> Insn: the instruction at pc (mnemonic in Zilog's
spelling, its size, where it goes: flow and targets), by the usual
x/y/z/p/q split of the opcode byte, with the CB, ED, DD/FD and DDCB/FDCB
pages.  Undocumented forms (IXH, SLL, the DDCB copies into a register)
decode too, so whatever the driver holds decodes.
"""
import argparse
import os
import sys

R = ['b', 'c', 'd', 'e', 'h', 'l', '(hl)', 'a']
RP = ['bc', 'de', 'hl', 'sp']
RP2 = ['bc', 'de', 'hl', 'af']
CC = ['nz', 'z', 'nc', 'c', 'po', 'pe', 'p', 'm']
ALU = ['add a,', 'adc a,', 'sub ', 'sbc a,', 'and ', 'xor ', 'or ', 'cp ']
ROT = ['rlc', 'rrc', 'rl', 'rr', 'sla', 'sra', 'sll', 'srl']
IM = ['0', '0', '1', '2', '0', '0', '1', '2']
BLI = {(4, 0): 'ldi', (4, 1): 'cpi', (4, 2): 'ini', (4, 3): 'outi',
       (5, 0): 'ldd', (5, 1): 'cpd', (5, 2): 'ind', (5, 3): 'outd',
       (6, 0): 'ldir', (6, 1): 'cpir', (6, 2): 'inir', (6, 3): 'otir',
       (7, 0): 'lddr', (7, 1): 'cpdr', (7, 2): 'indr', (7, 3): 'otdr'}


class Insn:
    """addr, size, text; flow: None (falls through), 'jump', 'branch'
    (conditional), 'call', 'ccall' (conditional call), 'ret', 'cret'
    (conditional return), 'jumpind' (jp (hl) and the like), 'halt';
    target: a jump's or call's address."""
    __slots__ = ('addr', 'size', 'text', 'flow', 'target', 'op')

    def __init__(self, addr, size, text, flow=None, target=None, op=None):
        self.addr = addr
        self.size = size
        self.text = text
        self.flow = flow
        self.target = target
        self.op = op

    def __repr__(self):
        return '%04X %s' % (self.addr, self.text)


def hx(v):
    return '0x%X' % v if v > 9 else '%d' % v


def decode(mem, pc):
    """The instruction at pc in mem (bytes-like, Z80 addresses)."""
    at = pc
    b = lambda: mem[at & 0xFFFF] if (at & 0xFFFF) < len(mem) else 0
    i = [0]

    def byte():
        v = mem[(pc + i[0]) & 0xFFFF] if ((pc + i[0]) & 0xFFFF) < len(mem) \
            else 0
        i[0] += 1
        return v

    def word():
        lo = byte()
        return lo | byte() << 8

    def sbyte():
        v = byte()
        return v - 256 if v >= 128 else v

    op = byte()
    idx = None
    if op in (0xDD, 0xFD):
        idx = 'ix' if op == 0xDD else 'iy'
        op = byte()
        if op in (0xDD, 0xFD, 0xED):
            # a prefix before another: a no-op of its own
            return Insn(pc, 1, 'nop | prefix 0x%02X' % mem[pc], op=mem[pc])
    first = op

    def r(n, d=None):
        """Register n, through ix/iy under a prefix ((hl) -> (ix+d),
        h/l -> ixh/ixl unless the instruction also has (ix+d))."""
        if idx is None:
            return R[n]
        if n == 6:
            return '(%s%+d)' % (idx, d)
        if d is None and n in (4, 5):
            return idx + 'hl'[n - 4]
        return R[n]

    def rp(n):
        return idx if (idx and n == 2) else RP[n]

    def rp2(n):
        return idx if (idx and n == 2) else RP2[n]

    def done(text, flow=None, target=None):
        return Insn(pc, i[0], text, flow, target, mem[pc])

    if op == 0xCB:
        if idx:
            d = sbyte()
            op = byte()
            x, y, z = op >> 6, (op >> 3) & 7, op & 7
            m = '(%s%+d)' % (idx, d)
            copy = '' if z == 6 else ',' + R[z]
            if x == 0:
                return done('%s %s%s' % (ROT[y], m, copy))
            if x == 1:
                return done('bit %d,%s' % (y, m))
            return done('%s %d,%s%s' % (['', '', 'res', 'set'][x], y, m,
                                        copy))
        op = byte()
        x, y, z = op >> 6, (op >> 3) & 7, op & 7
        if x == 0:
            return done('%s %s' % (ROT[y], R[z]))
        return done('%s %d,%s' % (['', 'bit', 'res', 'set'][x], y, R[z]))

    if op == 0xED:
        op = byte()
        x, y, z = op >> 6, (op >> 3) & 7, op & 7
        p, q = y >> 1, y & 1
        if x == 1:
            if z == 0:
                return done('in (c)' if y == 6 else 'in %s,(c)' % R[y])
            if z == 1:
                return done('out (c),0' if y == 6 else 'out (c),%s' % R[y])
            if z == 2:
                return done('%s hl,%s' % ('sbc' if q == 0 else 'adc', RP[p]))
            if z == 3:
                nn = word()
                if q == 0:
                    return done('ld (%s),%s' % (hx(nn), RP[p]))
                return done('ld %s,(%s)' % (RP[p], hx(nn)))
            if z == 4:
                return done('neg')
            if z == 5:
                return done('reti' if y == 1 else 'retn', 'ret')
            if z == 6:
                return done('im %s' % IM[y])
            return done(['ld i,a', 'ld r,a', 'ld a,i', 'ld a,r', 'rrd', 'rld',
                         'nop', 'nop'][y])
        if x == 2 and (y, z) in BLI:
            return done(BLI[(y, z)])
        return done('nop | ed 0x%02X' % op)

    x, y, z = op >> 6, (op >> 3) & 7, op & 7
    p, q = y >> 1, y & 1
    if x == 0:
        if z == 0:
            if y == 0:
                return done('nop')
            if y == 1:
                return done("ex af,af'")
            d = sbyte()
            t = (pc + i[0] + d) & 0xFFFF
            if y == 2:
                return done('djnz %s' % hx(t), 'branch', t)
            if y == 3:
                return done('jr %s' % hx(t), 'jump', t)
            return done('jr %s,%s' % (CC[y - 4], hx(t)), 'branch', t)
        if z == 1:
            if q == 0:
                return done('ld %s,%s' % (rp(p), hx(word())))
            return done('add %s,%s' % (rp(2), rp(p)))
        if z == 2:
            if q == 0:
                if p < 2:
                    return done('ld (%s),a' % RP[p])
                nn = word()
                return done(('ld (%s),%s' % (hx(nn), rp(2))) if p == 2
                            else 'ld (%s),a' % hx(nn))
            if p < 2:
                return done('ld a,(%s)' % RP[p])
            nn = word()
            return done(('ld %s,(%s)' % (rp(2), hx(nn))) if p == 2
                        else 'ld a,(%s)' % hx(nn))
        if z == 3:
            return done('%s %s' % ('inc' if q == 0 else 'dec', rp(p)))
        if z in (4, 5):
            d = sbyte() if (idx and y == 6) else None
            return done('%s %s' % ('inc' if z == 4 else 'dec', r(y, d)))
        if z == 6:
            d = sbyte() if (idx and y == 6) else None
            return done('ld %s,%s' % (r(y, d), hx(byte())))
        return done(['rlca', 'rrca', 'rla', 'rra', 'daa', 'cpl', 'scf',
                     'ccf'][y])
    if x == 1:
        if op == 0x76:
            return done('halt', 'halt')
        if idx and (y == 6 or z == 6):
            d = sbyte()
            # (ix+d) with the plain h/l on the other side
            dst = r(y, d) if y == 6 else R[y]
            src = r(z, d) if z == 6 else R[z]
            return done('ld %s,%s' % (dst, src))
        return done('ld %s,%s' % (r(y), r(z)))
    if x == 2:
        d = sbyte() if (idx and z == 6) else None
        return done(ALU[y] + r(z, d))
    # x == 3
    if z == 0:
        return done('ret %s' % CC[y], 'cret')
    if z == 1:
        if q == 0:
            return done('pop %s' % rp2(p))
        if p == 0:
            return done('ret', 'ret')
        if p == 1:
            return done('exx')
        if p == 2:
            return done('jp (%s)' % (idx or 'hl'), 'jumpind')
        return done('ld sp,%s' % (idx or 'hl'))
    if z == 2:
        nn = word()
        return done('jp %s,%s' % (CC[y], hx(nn)), 'branch', nn)
    if z == 3:
        if y == 0:
            nn = word()
            return done('jp %s' % hx(nn), 'jump', nn)
        if y == 2:
            return done('out (%s),a' % hx(byte()))
        if y == 3:
            return done('in a,(%s)' % hx(byte()))
        return done(['', '', '', '', 'ex (sp),%s' % (idx or 'hl'),
                     'ex de,hl', 'di', 'ei'][y])
    if z == 4:
        nn = word()
        return done('call %s,%s' % (CC[y], hx(nn)), 'ccall', nn)
    if z == 5:
        if q == 0:
            return done('push %s' % rp2(p))
        nn = word()
        return done('call %s' % hx(nn), 'call', nn)
    if z == 6:
        return done(ALU[y] + hx(byte()))
    return done('rst %s' % hx(y * 8), 'call', y * 8)


def walk(mem, roots):
    """Every instruction reached from roots by flow (not through jp (hl)
    or tables)."""
    seen = {}
    todo = list(roots)
    while todo:
        a = todo.pop()
        while a not in seen and a < len(mem):
            ins = decode(mem, a)
            seen[a] = ins
            if ins.target is not None and ins.target < len(mem):
                todo.append(ins.target)
            if ins.flow in ('jump', 'ret', 'jumpind', 'halt'):
                break
            a += ins.size
    return seen


DRIVER = 0xF8000     # us; its length is the word at +6 (func_00B7F4)


def driver(rom):
    n = rom[DRIVER + 6] | rom[DRIVER + 7] << 8
    return rom[DRIVER:DRIVER + n]


def write_ran(path, files):
    """zcode.VERSION.txt: where Z80 instructions started in any of the
    runs (tools/trace_all.sh; files: mdtrace --zcover's, 8 KB each)."""
    ran = bytearray(0x2000)
    for p in files:
        for i, b in enumerate(open(p, 'rb').read()[:0x2000]):
            if b:
                ran[i] = 1
    with open(path, 'w') as f:
        f.write('# Z80 addresses where an instruction ran in a trace '
                '(tools/trace_all.sh)\n')
        for i in range(0x2000):
            if ran[i]:
                f.write('%04X\n' % i)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('rom', nargs='?', default=os.path.join(
        os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
        'baserom.us.bin'))
    ap.add_argument('--at', action='append', default=[],
                    help='another root (hex)')
    ap.add_argument('--write', metavar='LIST',
                    help='write the Z80 code the traces ran (the '
                    'mdtrace --zcover files given) to LIST')
    ap.add_argument('zcover', nargs='*')
    a = ap.parse_args()
    if a.write:
        write_ran(a.write, [a.rom] + a.zcover)
        return 0
    mem = driver(open(a.rom, 'rb').read())
    roots = [0x0000, 0x0038, 0x0066] + [int(x, 16) for x in a.at]
    code = walk(mem, roots)
    for addr in sorted(code):
        ins = code[addr]
        raw = ' '.join('%02X' % mem[addr + k] for k in range(ins.size))
        print('%04X  %-12s  %s' % (addr, raw, ins.text))


if __name__ == '__main__':
    sys.exit(main())
