"""MC68000 instruction decoder for the disassembler (tools/disasm.py).

decode(rom, addr) returns an Insn or None.  None means the words at addr are
not an instruction that GNU as would assemble back to the same bytes: an
illegal or 68020-only encoding, a field the 68000 ignores but that is set
(index scale, a byte immediate's high byte), or one of the encodings as
picks differently from the same text and that the 68000 would not run
the same.  The disassembler keeps such words as data.  A real instruction
gas has no spelling for (`add.w #imm,d0`, which gas makes `addi`) is
decoded with raw=True: written as words, its text a comment.

The output assembles with `--register-prefix-optional` and `.opt noquick`
(else gas makes `move.l #0,d0` a moveq); tools/test_m68k.py checks every
encoding against gas.

Operands are Op objects; format() renders them in Motorola syntax for gas
(`--register-prefix-optional`), with a callback that may turn an address
into a symbol.
"""

import struct

SIZES = {0: 'b', 1: 'w', 2: 'l'}
CC = ['t', 'f', 'hi', 'ls', 'cc', 'cs', 'ne', 'eq',
      'vc', 'vs', 'pl', 'mi', 'ge', 'lt', 'gt', 'le']
# Bcc spells cc 0/1 as bra/bsr; DBcc 1 is dbra (gas accepts dbf too)
DBCC = ['dbt', 'dbra'] + ['db' + c for c in CC[2:]]


def hx(v):
    if v < 0:
        return '-0x%X' % -v
    return '0x%X' % v


class Op:
    """One operand.  kind is one of
    dn an ind postinc predec disp index absw absl pcdisp pcindex imm
    sr ccr usp reglist quick label.
    addr: the absolute address a memory operand or label names, if fixed
    (absw/absl/pcdisp/label; absw sign-extended to 24 bits).
    """
    __slots__ = ('kind', 'reg', 'val', 'xreg', 'xsize', 'addr', 'size',
                 'pre')

    def __init__(self, kind, reg=None, val=None, xreg=None, xsize=None,
                 addr=None, size=None, pre=False):
        self.kind = kind
        self.reg = reg
        self.val = val
        self.xreg = xreg
        self.xsize = xsize
        self.addr = addr
        self.size = size
        self.pre = pre

    def format(self, sym=None):
        """sym(addr, kind) -> name or None; kind is 'abs' (`(x).l`),
        'abs.w' (`(x).w`), 'pc', 'imm', 'branch' or 'disp' (a positive
        displacement off an address register).  sym is asked for any address this operand holds."""
        k = self.kind
        if k == 'dn':
            return 'd%d' % self.reg
        if k == 'an':
            return 'sp' if self.reg == 7 else 'a%d' % self.reg
        if k == 'ind':
            return '(%s)' % areg(self.reg)
        if k == 'postinc':
            return '(%s)+' % areg(self.reg)
        if k == 'predec':
            return '-(%s)' % areg(self.reg)
        if k == 'disp':
            # a displacement can be a table's address, the register an
            # index: sym(val, 'disp') decides
            name = sym(self.val, 'disp') if sym and self.val > 0 else None
            # gas turns (0,a0) into (a0); the size keeps the word
            return '(%s,%s)' % (name or (hx(self.val) if self.val else
                                         '0.w'), areg(self.reg))
        if k == 'index':
            return '(%s,%s,%s.%s)' % (hx(self.val), areg(self.reg),
                                      xname(self.xreg), self.xsize)
        if k in ('absw', 'absl'):
            # a name only where the operand is the plain address
            name = sym(self.addr, 'abs' if k == 'absl' else 'abs.w') \
                if sym and (k == 'absw' or self.val == self.addr) else None
            return '(%s).%s' % (name or hx(self.val),
                                'w' if k == 'absw' else 'l')
        if k == 'pcdisp':
            name = sym(self.addr, 'pc') if sym else None
            if name is None:
                return '(%s,pc)' % hx(self.val)
            return '(%s,pc)' % name
        if k == 'pcindex':
            name = sym(self.addr, 'pc') if sym else None
            return '(%s,pc,%s.%s)' % (name or hx(self.val),
                                      xname(self.xreg), self.xsize)
        if k == 'imm':
            name = sym(self.val, 'imm') if (sym and self.size == 'l') \
                else None
            return '#' + (name or hx(self.val))
        if k == 'quick':
            return '#%d' % self.val
        if k in ('sr', 'ccr', 'usp'):
            return k
        if k == 'reglist':
            return reglist(self.val, self.pre)
        if k == 'label':
            name = sym(self.addr, 'branch') if sym else None
            return name or hx(self.addr)
        raise ValueError(k)


def areg(n):
    return 'sp' if n == 7 else 'a%d' % n


def xname(x):
    return ('a%d' % (x - 8) if x != 15 else 'sp') if x >= 8 else 'd%d' % x


def reglist(mask, pre):
    """movem mask -> 'd0-d3/a0/a6'.  For -(An) the mask is bit-reversed."""
    if pre:
        mask = int('{:016b}'.format(mask)[::-1], 2)
    names = ['d%d' % i for i in range(8)] + ['a%d' % i for i in range(8)]
    out = []
    i = 0
    while i < 16:
        if mask >> i & 1:
            j = i
            while j + 1 < 16 and (j + 1) // 8 == i // 8 and mask >> (j + 1) & 1:
                j += 1
            out.append(names[i] if i == j else '%s-%s' % (names[i], names[j]))
            i = j + 1
        else:
            i += 1
    return '/'.join(out)


class Insn:
    __slots__ = ('addr', 'size', 'mn', 'ops', 'flow', 'target', 'raw')

    def __init__(self, addr, size, mn, ops, flow=None, target=None,
                 raw=False):
        self.addr = addr
        # raw: a real instruction gas has no spelling for (it would pick a
        # shorter or other encoding); written as words, the text a comment
        self.raw = raw
        self.size = size
        self.mn = mn
        self.ops = ops
        # flow: None (falls through), 'jump' (no fallthrough), 'branch'
        # (conditional), 'call', 'ret' (rts/rte/rtr), 'stop'
        self.flow = flow
        self.target = target

    def format(self, sym=None):
        if not self.ops:
            return self.mn
        return '%s %s' % (self.mn, ','.join(o.format(sym) for o in self.ops))

    def refs(self):
        """(addr, kind) of every fixed address this instruction names."""
        out = []
        for o in self.ops:
            if o.kind in ('absw', 'absl', 'pcdisp', 'pcindex', 'label'):
                out.append((o.addr, o.kind))
            elif o.kind == 'imm' and o.size == 'l':
                out.append((o.val, 'imm'))
        return out


class Bad(Exception):
    pass


class Reader:
    def __init__(self, rom, addr):
        self.rom = rom
        self.start = addr
        self.pos = addr

    def word(self):
        if self.pos + 2 > len(self.rom):
            raise Bad()
        v = struct.unpack_from('>H', self.rom, self.pos)[0]
        self.pos += 2
        return v

    def sword(self):
        v = self.word()
        return v - 0x10000 if v & 0x8000 else v

    def long(self):
        hi = self.word()
        return hi << 16 | self.word()


def s8(v):
    return v - 0x100 if v & 0x80 else v


def ea(r, mode, reg, size, allowed='all'):
    """Decode an effective address.  size is 'b'/'w'/'l' for immediates.
    allowed: a string of class letters
      d Dn, a An, m (An), + (An)+, - -(An), o (d16,An), x (d8,An,Xn),
      w abs.w, l abs.l, p (d16,PC), q (d8,PC,Xn), i #imm
    """
    if allowed == 'all':
        allowed = 'dam+-oxwlpqi'
    if mode == 7:
        cls = 'wlpqi'[reg] if reg < 5 else None
    else:
        cls = 'dam+-ox'[mode]
    if cls is None or cls not in allowed:
        raise Bad()
    if cls == 'd':
        return Op('dn', reg)
    if cls == 'a':
        return Op('an', reg)
    if cls == 'm':
        return Op('ind', reg)
    if cls == '+':
        return Op('postinc', reg)
    if cls == '-':
        return Op('predec', reg)
    if cls == 'o':
        return Op('disp', reg, r.sword())
    if cls in 'xq':
        base = r.pos
        ext = r.word()
        # brief format only, scale 0 (68000 ignores bits 8-10; gas writes 0)
        if ext & 0x0700:
            raise Bad()
        x = ext >> 12
        xs = 'l' if ext & 0x800 else 'w'
        d = s8(ext & 0xff)
        if cls == 'x':
            return Op('index', reg, d, x, xs)
        if d == 0:
            raise Bad()            # gas refuses a pc index to its own word
        return Op('pcindex', None, d, x, xs, addr=(base + d) & 0xffffff)
    if cls == 'w':
        v = r.sword()
        return Op('absw', None, v & 0xffff, addr=v & 0xffffff)
    if cls == 'l':
        v = r.long()
        # the 68000's bus is 24 bits: (0xFFFF0000).l is RAM at 0xFF0000
        return Op('absl', None, v, addr=v & 0xffffff)
    if cls == 'p':
        base = r.pos
        d = r.sword()
        return Op('pcdisp', None, d, addr=(base + d) & 0xffffff)
    if cls == 'i':
        if size == 'b':
            v = r.word()
            if v & 0xff00:
                raise Bad()
            return Op('imm', None, v, size='b')
        if size == 'w':
            return Op('imm', None, r.word(), size='w')
        return Op('imm', None, r.long(), size='l')
    raise Bad()


DATA_ALT = 'd+-oxwl' + 'm'        # data alterable
MEM_ALT = 'm+-oxwl'               # memory alterable
DATA = 'dm+-oxwlpqi'              # data addressing
CTRL = 'moxwlpq'                  # control
ALT = 'dam+-oxwl'                 # alterable


def decode(rom, addr):
    if addr & 1 or addr + 2 > len(rom):
        return None
    r = Reader(rom, addr)
    try:
        ins = _decode(r, addr)
    except Bad:
        return None
    if ins is None:
        return None
    ins.size = r.pos - addr
    return ins


def _decode(r, addr):
    w = r.word()
    top = w >> 12
    reg9 = (w >> 9) & 7
    opm = (w >> 6) & 7
    mode = (w >> 3) & 7
    reg = w & 7
    sz2 = (w >> 6) & 3
    I = Insn

    if top == 0:
        if w & 0x100:
            if mode == 1:          # movep
                d = r.sword()
                mn = 'movep.' + ('l' if w & 0x40 else 'w')
                mem = Op('disp', reg, d)
                if w & 0x80:
                    return I(addr, 0, mn, [Op('dn', reg9), mem])
                return I(addr, 0, mn, [mem, Op('dn', reg9)])
            mn = ['btst', 'bchg', 'bclr', 'bset'][sz2]
            allowed = DATA if sz2 == 0 else DATA_ALT
            if sz2 == 0:
                allowed = 'dm+-oxwlpqi'
            dst = ea(r, mode, reg, 'b', allowed)
            mn += '.l' if mode == 0 else '.b'
            return I(addr, 0, mn, [Op('dn', reg9), dst])
        if reg9 == 4:              # static bit ops
            n = r.word()
            if n & 0xff00:
                raise Bad()
            mn = ['btst', 'bchg', 'bclr', 'bset'][sz2]
            allowed = 'dm+-oxwlpq' if sz2 == 0 else DATA_ALT
            dst = ea(r, mode, reg, 'b', allowed)
            # gas accepts any count for memory and masks nothing; keep
            # counts the 68000 would honour
            if mode == 0 and n > 31 or mode != 0 and n > 7:
                raise Bad()
            mn += '.l' if mode == 0 else '.b'
            return I(addr, 0, mn, [Op('imm', None, n, size='b'), dst])
        names = {0: 'ori', 1: 'andi', 2: 'subi', 3: 'addi', 5: 'eori',
                 6: 'cmpi'}
        if reg9 not in names or sz2 == 3:
            raise Bad()
        mn = names[reg9]
        if mode == 7 and reg == 4 and reg9 in (0, 1, 5):
            # to CCR (size b) / SR (size w)
            if sz2 == 0:
                v = r.word()
                if v & 0xff00:
                    raise Bad()
                return I(addr, 0, mn, [Op('imm', None, v, size='b'),
                                       Op('ccr')])
            if sz2 == 1:
                return I(addr, 0, mn, [Op('imm', None, r.word(), size='w'),
                                       Op('sr')])
            raise Bad()
        s = SIZES[sz2]
        src = ea(r, 7, 4, s)
        allowed = DATA_ALT
        if reg9 == 6:
            allowed = DATA_ALT     # 68000: cmpi has no pc modes
        dst = ea(r, mode, reg, s, allowed)
        return I(addr, 0, '%s.%s' % (mn, s), [src, dst])

    if top in (1, 2, 3):
        s = {1: 'b', 3: 'w', 2: 'l'}[top]
        src = ea(r, mode, reg, s, 'all' if s != 'b' else 'dm+-oxwlpqi')
        dmode = opm
        if dmode == 1:
            if s == 'b':
                raise Bad()
            return I(addr, 0, 'movea.' + s, [src, Op('an', reg9)])
        dst = ea(r, dmode, reg9, s, DATA_ALT)
        return I(addr, 0, 'move.' + s, [src, dst])

    if top == 4:
        return _decode4(r, addr, w, reg9, opm, mode, reg, sz2)

    if top == 5:
        if sz2 == 3:
            cc = (w >> 8) & 0xf
            if mode == 1:
                d = r.sword()
                tgt = (addr + 2 + d) & 0xffffff
                return I(addr, 0, DBCC[cc],
                         [Op('dn', reg), Op('label', addr=tgt)],
                         flow='branch', target=tgt)
            dst = ea(r, mode, reg, 'b', DATA_ALT)
            return I(addr, 0, 's' + CC[cc], [dst])
        s = SIZES[sz2]
        q = reg9 or 8
        dst = ea(r, mode, reg, s, ALT if s != 'b' else DATA_ALT)
        mn = 'subq' if w & 0x100 else 'addq'
        return I(addr, 0, '%s.%s' % (mn, s), [Op('quick', val=q), dst])

    if top == 6:
        cc = (w >> 8) & 0xf
        d = w & 0xff
        if d == 0:
            d = r.sword()
            sfx = 'w'
            tgt = (addr + 2 + d) & 0xffffff
        else:
            if d == 0xff:          # 68020 long branch
                raise Bad()
            sfx = 's'
            tgt = (addr + 2 + s8(d)) & 0xffffff
        if tgt & 1:
            raise Bad()
        if cc == 0:
            return I(addr, 0, 'bra.' + sfx, [Op('label', addr=tgt)],
                     flow='jump', target=tgt)
        if cc == 1:
            return I(addr, 0, 'bsr.' + sfx, [Op('label', addr=tgt)],
                     flow='call', target=tgt)
        return I(addr, 0, 'b%s.%s' % (CC[cc], sfx), [Op('label', addr=tgt)],
                 flow='branch', target=tgt)

    if top == 7:
        if w & 0x100:
            raise Bad()
        return I(addr, 0, 'moveq', [Op('imm', None, s8(w & 0xff), size='q'),
                                    Op('dn', reg9)])

    if top in (8, 0xC):
        if opm in (3, 7):
            mn = ({8: 'divu', 0xC: 'mulu'} if opm == 3 else
                  {8: 'divs', 0xC: 'muls'})[top] + '.w'
            src = ea(r, mode, reg, 'w', DATA)
            return I(addr, 0, mn, [src, Op('dn', reg9)])
        if opm == 4 and mode in (0, 1):
            mn = 'sbcd' if top == 8 else 'abcd'
            if mode == 0:
                return I(addr, 0, mn, [Op('dn', reg), Op('dn', reg9)])
            return I(addr, 0, mn, [Op('predec', reg), Op('predec', reg9)])
        if top == 0xC and opm in (5, 6) and mode in (0, 1):
            # exg
            if opm == 5 and mode == 0:
                return I(addr, 0, 'exg', [Op('dn', reg9), Op('dn', reg)])
            if opm == 5 and mode == 1:
                return I(addr, 0, 'exg', [Op('an', reg9), Op('an', reg)])
            raise Bad()
        if top == 0xC and opm == 6 and mode == 1:
            return I(addr, 0, 'exg', [Op('dn', reg9), Op('an', reg)])
        mn = 'or' if top == 8 else 'and'
        s = SIZES[opm & 3]
        if opm < 4:
            src = ea(r, mode, reg, s, DATA)
            # with an immediate, gas assembles ori/andi instead
            return I(addr, 0, '%s.%s' % (mn, s), [src, Op('dn', reg9)],
                     raw=src.kind == 'imm')
        dst = ea(r, mode, reg, s, MEM_ALT)
        return I(addr, 0, '%s.%s' % (mn, s), [Op('dn', reg9), dst])

    if top in (9, 0xD):
        mn = 'sub' if top == 9 else 'add'
        if opm in (3, 7):
            s = 'w' if opm == 3 else 'l'
            src = ea(r, mode, reg, s)
            return I(addr, 0, '%sa.%s' % (mn, s), [src, Op('an', reg9)])
        s = SIZES[opm & 3]
        if opm >= 4 and mode in (0, 1):
            mn += 'x.' + s
            if mode == 0:
                return I(addr, 0, mn, [Op('dn', reg), Op('dn', reg9)])
            return I(addr, 0, mn, [Op('predec', reg), Op('predec', reg9)])
        if opm < 4:
            src = ea(r, mode, reg, s, 'all' if s != 'b' else DATA)
            # with an immediate, gas assembles addi/subi instead
            return I(addr, 0, '%s.%s' % (mn, s), [src, Op('dn', reg9)],
                     raw=src.kind == 'imm')
        dst = ea(r, mode, reg, s, MEM_ALT)
        return I(addr, 0, '%s.%s' % (mn, s), [Op('dn', reg9), dst])

    if top == 0xB:
        if opm in (3, 7):
            s = 'w' if opm == 3 else 'l'
            src = ea(r, mode, reg, s)
            return I(addr, 0, 'cmpa.' + s, [src, Op('an', reg9)])
        s = SIZES[opm & 3]
        if opm < 4:
            src = ea(r, mode, reg, s, 'all' if s != 'b' else DATA)
            # with an immediate, gas assembles cmpi instead
            return I(addr, 0, 'cmp.' + s, [src, Op('dn', reg9)],
                     raw=src.kind == 'imm')
        if mode == 1:
            return I(addr, 0, 'cmpm.' + s,
                     [Op('postinc', reg), Op('postinc', reg9)])
        dst = ea(r, mode, reg, s, DATA_ALT)
        return I(addr, 0, 'eor.' + s, [Op('dn', reg9), dst])

    if top == 0xE:
        names = ['as', 'ls', 'rox', 'ro']
        dr = 'l' if w & 0x100 else 'r'
        if sz2 == 3:
            kind = (w >> 9) & 7
            if kind > 3:
                raise Bad()
            dst = ea(r, mode, reg, 'w', MEM_ALT)
            return I(addr, 0, '%s%s.w' % (names[kind], dr), [dst])
        s = SIZES[sz2]
        kind = (w >> 3) & 3
        mn = '%s%s.%s' % (names[kind], dr, s)
        if w & 0x20:
            return I(addr, 0, mn, [Op('dn', reg9), Op('dn', reg)])
        return I(addr, 0, mn, [Op('quick', val=reg9 or 8), Op('dn', reg)])

    raise Bad()                    # line A, line F


def _decode4(r, addr, w, reg9, opm, mode, reg, sz2):
    I = Insn
    if w == 0x4e71:
        return I(addr, 0, 'nop', [])
    if w == 0x4e75:
        return I(addr, 0, 'rts', [], flow='ret')
    if w == 0x4e73:
        return I(addr, 0, 'rte', [], flow='ret')
    if w == 0x4e77:
        return I(addr, 0, 'rtr', [], flow='ret')
    if w == 0x4e70:
        return I(addr, 0, 'reset', [])
    if w == 0x4e76:
        return I(addr, 0, 'trapv', [])
    if w == 0x4e72:
        return I(addr, 0, 'stop', [Op('imm', None, r.word(), size='w')])
    if w == 0x4afc:
        return I(addr, 0, 'illegal', [], flow='ret')
    if w & 0xfff0 == 0x4e40:
        return I(addr, 0, 'trap', [Op('quick', val=w & 0xf)])
    if w & 0xfff8 == 0x4e50:
        return I(addr, 0, 'link', [Op('an', reg), Op('imm', None, r.sword(),
                                                     size='w')])
    if w & 0xfff8 == 0x4e58:
        return I(addr, 0, 'unlk', [Op('an', reg)])
    if w & 0xfff0 == 0x4e60:
        if w & 8:
            return I(addr, 0, 'move.l', [Op('usp'), Op('an', reg)])
        return I(addr, 0, 'move.l', [Op('an', reg), Op('usp')])
    if w & 0xffc0 == 0x4e80:
        dst = ea(r, mode, reg, 'l', CTRL)
        tgt = dst.addr if dst.kind in ('absw', 'absl', 'pcdisp') else None
        return I(addr, 0, 'jsr', [dst], flow='call', target=tgt)
    if w & 0xffc0 == 0x4ec0:
        dst = ea(r, mode, reg, 'l', CTRL)
        tgt = dst.addr if dst.kind in ('absw', 'absl', 'pcdisp') else None
        return I(addr, 0, 'jmp', [dst], flow='jump', target=tgt)
    if w & 0xfff8 == 0x4840:
        return I(addr, 0, 'swap', [Op('dn', reg)])
    if w & 0xffb8 == 0x4880:
        return I(addr, 0, 'ext.' + ('l' if w & 0x40 else 'w'),
                 [Op('dn', reg)])
    if w & 0xfb80 == 0x4880:       # movem
        s = 'l' if w & 0x40 else 'w'
        mask = r.word()
        if not mask:
            raise Bad()        # an empty list has no gas spelling
        if w & 0x400:              # mem -> regs
            src = ea(r, mode, reg, s, 'm+oxwlpq')
            return I(addr, 0, 'movem.' + s,
                     [src, Op('reglist', val=mask)])
        dst = ea(r, mode, reg, s, 'm-oxwl')
        return I(addr, 0, 'movem.' + s,
                 [Op('reglist', val=mask, pre=mode == 4), dst])
    if w & 0xf1c0 == 0x41c0:
        src = ea(r, mode, reg, 'l', CTRL)
        return I(addr, 0, 'lea', [src, Op('an', reg9)])
    if w & 0xf1c0 == 0x4180:
        src = ea(r, mode, reg, 'w', DATA)
        return I(addr, 0, 'chk.w', [src, Op('dn', reg9)])
    if w & 0xffc0 == 0x4840:
        src = ea(r, mode, reg, 'l', CTRL)
        return I(addr, 0, 'pea', [src])
    if w & 0xffc0 == 0x4800:
        return I(addr, 0, 'nbcd', [ea(r, mode, reg, 'b', DATA_ALT)])
    if w & 0xffc0 == 0x40c0:
        return I(addr, 0, 'move.w', [Op('sr'), ea(r, mode, reg, 'w',
                                                  DATA_ALT)])
    if w & 0xffc0 == 0x44c0:
        return I(addr, 0, 'move.w', [ea(r, mode, reg, 'w', DATA),
                                     Op('ccr')])
    if w & 0xffc0 == 0x46c0:
        return I(addr, 0, 'move.w', [ea(r, mode, reg, 'w', DATA),
                                     Op('sr')])
    if w & 0xffc0 == 0x4ac0:
        return I(addr, 0, 'tas', [ea(r, mode, reg, 'b', DATA_ALT)])
    one = {0x40: 'negx', 0x42: 'clr', 0x44: 'neg', 0x46: 'not', 0x4a: 'tst'}
    hi = w >> 8
    if hi in one and sz2 != 3:
        s = SIZES[sz2]
        dst = ea(r, mode, reg, s, DATA_ALT)
        return I(addr, 0, '%s.%s' % (one[hi], s), [dst])
    raise Bad()
