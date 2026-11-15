"""How long each instruction takes, in 68000 cycles, near enough
(tools/recomp; docs/PORT.md "Time").

The port lets time pass a basic block at a time (`self.tick(N)`): the
VDP draws its lines and the interrupts come as they would, so code that
runs for several frames without waiting (decompression, loading) lets the
vertical interrupt in as on the console.  The figures follow the 68000's
timing tables in outline: 4 cycles a bus access (the opcode word, each
extension word, each word of an operand read or written) and the extra
the ALU, shifts, multiplication and division take.
"""

EA_WORDS = {'dn': 0, 'an': 0, 'ind': 0, 'postinc': 0, 'predec': 0,
            'disp': 1, 'index': 1, 'absw': 1, 'absl': 2, 'pcdisp': 1,
            'pcindex': 1, 'imm': 1, 'quick': 0, 'label': 0, 'reglist': 1,
            'sr': 0, 'ccr': 0, 'usp': 0}
EXTRA = {'predec': 2, 'index': 2, 'pcindex': 2}


def operand_cycles(o, size, read=True):
    """Bus cycles to read (or write) operand o at size ('b'/'w'/'l')."""
    k = o.kind
    words = EA_WORDS.get(k, 0)
    if k == 'imm':
        words = 2 if size == 'l' else 1
        return 4 * words
    t = 4 * words + EXTRA.get(k, 0)
    if k in ('ind', 'postinc', 'predec', 'disp', 'index', 'absw', 'absl',
             'pcdisp', 'pcindex'):
        t += 8 if size == 'l' else 4
    return t


def cycles(i):
    mn = i.mn
    base, _, size = mn.partition('.')
    size = size or 'w'
    ops = i.ops
    t = 4                               # the opcode
    if base in ('bra', 'bsr') or (base[0] == 'b' and len(ops) == 1 and
                                  ops[0].kind == 'label'):
        return 10 + (8 if base == 'bsr' else 0)
    if base.startswith('db'):
        return 10                        # taken (the loop goes on)
    if base in ('jsr', 'jmp'):
        return {'absw': 10, 'absl': 12, 'ind': 8}.get(ops[0].kind, 14) + \
            (8 if base == 'jsr' else 0)
    if base in ('rts', 'rte'):
        return 16 if base == 'rts' else 20
    if base == 'movem':
        n = bin((ops[0] if ops[0].kind == 'reglist' else ops[1]).val).count(
            '1')
        mem = ops[1] if ops[0].kind == 'reglist' else ops[0]
        return 12 + n * (8 if size == 'l' else 4) + \
            4 * EA_WORDS.get(mem.kind, 0)
    if base == 'mulu':
        return 54 + operand_cycles(ops[0], 'w')
    if base == 'divu':
        return 120 + operand_cycles(ops[0], 'w')
    if base in ('asl', 'asr', 'lsl', 'lsr', 'rol', 'ror', 'roxl', 'roxr'):
        if len(ops) == 1:
            return 8 + operand_cycles(ops[0], 'w') * 2
        n = ops[0].val if ops[0].kind in ('quick', 'imm') else 4
        return (8 if size == 'l' else 6) + 2 * (n or 8)
    if base in ('lea', 'pea'):
        return 4 + 4 * EA_WORDS.get(ops[0].kind, 0) + (8 if base == 'pea'
                                                      else 0)
    if base in ('link', 'unlk'):
        return 16 if base == 'link' else 12
    if base in ('swap', 'ext', 'exg', 'nop', 'moveq'):
        return 4 if base != 'exg' else 6
    # the rest: read the source, read (and write) the destination
    if not ops:
        return t
    src = ops[0]
    t += operand_cycles(src, size)
    if len(ops) > 1:
        dst = ops[1]
        if dst.kind not in ('dn', 'an', 'sr', 'ccr', 'usp'):
            write = operand_cycles(dst, size)
            # a read-modify-write reads it too, but move and clr
            if base not in ('move', 'movea', 'clr'):
                write += 8 if size == 'l' else 4
            t += write
        elif size == 'l' and base not in ('move', 'movea', 'moveq'):
            t += 4
    elif base in ('clr', 'tst', 'neg', 'not', 'scc') or base[0] == 's':
        if src.kind not in ('dn', 'an'):
            t += 4                       # written back
    if base in ('btst', 'bset', 'bclr', 'bchg'):
        # a bit test reads its operand (8 + the address), a change reads
        # and writes it (12 +); on a register 6-10
        dst = ops[1]
        imm = ops[0].kind == 'imm'
        if dst.kind == 'dn':
            return (10 if imm else 6) + (2 if base != 'btst' else 0)
        ea = 4 * EA_WORDS.get(dst.kind, 0) + EXTRA.get(dst.kind, 0)
        return (8 if base == 'btst' else 12) + (4 if imm else 0) + ea
    if base == 'clr' and src.kind not in ('dn', 'an'):
        t += 4                           # the 68000 reads before clearing
    if base in ('addq', 'subq') and len(ops) > 1 and ops[1].kind == 'an':
        t = 8
    return t
