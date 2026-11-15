"""How long each instruction takes, in 68000 cycles (tools/recomp;
docs/PORT.md "Time").

The 68000's timing tables (the M68000 user manual, section 8), as Genesis
Plus GX's 68000 counts them, which tools/timing_check.py holds this to.
An instruction's time comes in up to three parts, in the order the
console spends them:

- `pre`: before a long instruction checks the bus refresh (MOVEM's
  registers);
- `dyn`: what depends on the data, worked out as it runs (a branch taken
  or not, a shift by a register, MULU/MULS/DIVU/DIVS, a bit above 15,
  Scc true);
- `base`: the rest, spent last.

`skip`: the instruction is long enough that a bus refresh due during it
is skipped (a shift by a register, MOVEM.L, DIVU/DIVS).
"""

# effective-address time (calculation and read): byte/word, long
EA = {'dn': (0, 0), 'an': (0, 0), 'ind': (4, 8), 'postinc': (4, 8),
      'predec': (6, 10), 'disp': (8, 12), 'index': (10, 14),
      'absw': (8, 12), 'absl': (12, 16), 'pcdisp': (8, 12),
      'pcindex': (10, 14), 'imm': (4, 8)}
# MOVE's destination: byte/word, long
MOVE_DST = {'dn': (0, 0), 'an': (0, 0), 'ind': (4, 8), 'postinc': (4, 8),
            'predec': (4, 8), 'disp': (8, 12), 'index': (10, 14),
            'absw': (8, 12), 'absl': (12, 16)}
# JMP, JSR, LEA, PEA by addressing mode
JMP = {'ind': 8, 'disp': 10, 'index': 14, 'absw': 10, 'absl': 12,
       'pcdisp': 10, 'pcindex': 14}
LEA = {'ind': 4, 'disp': 8, 'index': 12, 'absw': 8, 'absl': 12,
       'pcdisp': 8, 'pcindex': 12}
# MOVEM's fixed part: memory to registers, registers to memory
MOVEM_R = {'ind': 12, 'postinc': 12, 'disp': 16, 'index': 18, 'absw': 16,
           'absl': 20, 'pcdisp': 16, 'pcindex': 18}
MOVEM_W = {'ind': 8, 'predec': 8, 'disp': 12, 'index': 14, 'absw': 12,
           'absl': 16}
SHIFTS = ('asl', 'asr', 'lsl', 'lsr', 'rol', 'ror', 'roxl', 'roxr')
COND = ('t', 'f', 'hi', 'ls', 'cc', 'cs', 'ne', 'eq', 'vc', 'vs', 'pl', 'mi',
        'ge', 'lt', 'gt', 'le', 'hs', 'lo', 'ra')


class Cost:
    __slots__ = ('base', 'pre', 'dyn', 'skip')

    def __init__(self, base, pre=0, dyn=None, skip=False):
        self.base, self.pre, self.dyn, self.skip = base, pre, dyn, skip

    def __repr__(self):
        return 'Cost(%d, pre=%d, dyn=%r, skip=%r)' % (
            self.base, self.pre, self.dyn, self.skip)


def ea(o, size):
    t = EA.get(o.kind, (0, 0))
    return t[1] if size == 'l' else t[0]


def reg(o):
    return o.kind in ('dn', 'an')


def cost(i):
    mn = i.mn
    base, _, size = mn.partition('.')
    size = size or 'w'
    ops = i.ops
    k = [o.kind for o in ops]
    long = size == 'l'

    # flow
    if base in ('bra', 'bsr') and len(ops) == 1:
        return Cost(18 if base == 'bsr' else 10)
    if base[0] == 'b' and base[1:] in COND and len(ops) == 1:
        # not taken: 8 short, 12 word; taken 10
        return Cost(8 if size == 's' else 10,
                    dyn='bcc.s' if size == 's' else 'bcc.w')
    if base.startswith('db') and base[2:] in COND:
        # looping 10, condition true 12, count run out 14
        return Cost(10, dyn='dbcc')
    if base == 'jmp':
        return Cost(JMP[k[0]])
    if base == 'jsr':
        return Cost(JMP[k[0]] + 8)
    if base == 'rts':
        return Cost(16)
    if base in ('rte', 'rtr'):
        return Cost(20)
    if base == 'lea':
        return Cost(LEA[k[0]])
    if base == 'pea':
        return Cost(LEA[k[0]] + 8)
    if base == 'link':
        return Cost(16)
    if base == 'unlk':
        return Cost(12)
    if base in ('swap', 'ext', 'nop', 'moveq', 'illegal'):
        return Cost(4)
    if base == 'exg':
        return Cost(6)
    if base == 'trap':
        return Cost(34)
    if base == 'reset':
        return Cost(132)
    if base == 'stop':
        return Cost(4)

    # status register
    if base in ('andi', 'ori', 'eori') and k[1] in ('sr', 'ccr'):
        return Cost(20)
    if base == 'move' and k[0] == 'sr':
        return Cost(6 if reg(ops[1]) else 8 + ea(ops[1], 'w'))
    if base == 'move' and k[1] in ('sr', 'ccr'):
        return Cost(12 + ea(ops[0], 'w'))
    if base == 'move' and 'usp' in k:
        return Cost(4)

    if base == 'movem':
        regs, mem = (ops[0], ops[1]) if k[0] == 'reglist' else (ops[1],
                                                                 ops[0])
        n = bin(regs.val).count('1')
        fixed = (MOVEM_W if k[0] == 'reglist' else MOVEM_R)[mem.kind]
        return Cost(fixed, pre=n * (8 if long else 4), skip=long)

    if base in ('move', 'movea'):
        s = ea(ops[0], size)
        d = MOVE_DST[k[1]][1 if long else 0]
        return Cost(4 + s + d)

    if base in ('mulu', 'muls'):
        return Cost(ea(ops[0], 'w'), dyn=base)
    if base in ('divu', 'divs'):
        return Cost(ea(ops[0], 'w'), dyn=base, skip=True)

    if base in SHIFTS:
        if len(ops) == 1:                # memory, by one
            return Cost(8 + ea(ops[0], 'w'))
        fixed = 8 if long else 6
        if k[0] == 'dn':                 # by a register's count
            return Cost(fixed, dyn='shift', skip=True)
        return Cost(fixed + 2 * (ops[0].val or 8))

    if base in ('btst', 'bchg', 'bclr', 'bset'):
        imm = k[0] == 'imm'
        if k[1] == 'dn':
            t = {'btst': 6, 'bchg': 6, 'bset': 6, 'bclr': 8}[base] + \
                (4 if imm else 0)
            if imm:
                return Cost(t + (2 if ops[0].val & 31 >= 16 else 0))
            return Cost(t, dyn='bit')
        t = 4 if base == 'btst' else 8
        return Cost(t + (4 if imm else 0) + ea(ops[1], 'b'))

    if base == 'tst':
        return Cost(4 + ea(ops[0], size))
    if base in ('clr', 'neg', 'negx', 'not'):
        if reg(ops[0]):
            return Cost(6 if long else 4)
        return Cost((12 if long else 8) + ea(ops[0], size))
    if base[0] == 's' and base[1:] in COND:
        if reg(ops[0]):
            return Cost(4, dyn='scc')
        return Cost(8 + ea(ops[0], 'b'))
    if base == 'tas':
        return Cost(4 if reg(ops[0]) else 14 + ea(ops[0], 'b'))
    if base == 'nbcd':
        return Cost(6 if reg(ops[0]) else 8 + ea(ops[0], 'b'))
    if base in ('abcd', 'sbcd'):
        return Cost(6 if k[0] == 'dn' else 18)
    if base in ('addx', 'subx'):
        if k[0] == 'dn':
            return Cost(8 if long else 4)
        return Cost(30 if long else 18)
    if base == 'cmpm':
        return Cost(20 if long else 12)
    if base == 'movep':
        return Cost(24 if long else 16)
    if base == 'chk':
        return Cost(10 + ea(ops[0], 'w'))

    if base in ('addq', 'subq'):
        if k[1] == 'an':
            return Cost(8)
        if k[1] == 'dn':
            return Cost(8 if long else 4)
        return Cost((12 if long else 8) + ea(ops[1], size))
    if base in ('addi', 'subi', 'andi', 'ori', 'eori'):
        if k[1] == 'dn':
            return Cost(16 if long else 8)
        return Cost((20 if long else 12) + ea(ops[1], size))
    if base == 'cmpi':
        if k[1] == 'dn':
            return Cost(14 if long else 8)
        return Cost((12 if long else 8) + ea(ops[1], size))
    if base in ('adda', 'suba'):
        if long:
            return Cost((8 if k[0] in ('dn', 'an', 'imm') else 6) +
                        ea(ops[0], 'l'))
        return Cost(8 + ea(ops[0], 'w'))
    if base == 'cmpa':
        return Cost(6 + ea(ops[0], size))
    if base in ('add', 'sub', 'and', 'or', 'eor', 'cmp'):
        if k[1] in ('dn', 'an') and base != 'eor':
            if base == 'cmp':
                return Cost((6 if long else 4) + ea(ops[0], size))
            if long:
                return Cost((8 if k[0] in ('dn', 'an', 'imm') else 6) +
                            ea(ops[0], size))
            return Cost(4 + ea(ops[0], size))
        if k[1] == 'dn':                 # eor Dn,Dn
            return Cost(8 if long else 4)
        return Cost((12 if long else 8) + ea(ops[1], size))
    raise ValueError('no timing for %s' % mn)


def cycles(i):
    """The time without what depends on the data (the least it takes;
    a branch not taken)."""
    c = cost(i)
    return c.base + c.pre
