"""Instructions as statements over named values (tools/recomp).

Each instruction becomes assignments to the registers (`d0`..`d7`,
`a0`..`a6`, all u32), the condition flags (`x n z v c`, bool), stack
slots and temporaries, and loads and stores of memory, plus how control
leaves it.  Expressions are Rust text with what they read, so the
liveness pass (dataflow.py) can drop what nothing reads (most flags) and
the printer can write the rest as it is.

The stack is no memory here: stack.py knows sp's depth below the
routine's entry at each instruction, so a push is an assignment to a slot
local (`s8` is the slot 8 bytes below the entry), a pop reads it, and a
read above the entry is an argument the caller pushed (`arg4`: 4 bytes
above the return address).  `movem` to and from the stack saves and
restores registers through slots, which the printer turns into nothing
where the value comes back unchanged.
"""

from recomp import timing

REGS = ['d%d' % k for k in range(8)] + ['a%d' % k for k in range(7)]
FLAGS = ['x', 'n', 'z', 'v', 'c']
TY = {'b': 'u8', 'w': 'u16', 'l': 'u32'}
BITS = {'b': 8, 'w': 16, 'l': 32}
SIGNED = {'b': 'i8', 'w': 'i16', 'l': 'i32'}


class LowerError(Exception):
    pass


class X:
    """A Rust expression: its text, its type, what it reads, whether it
    has a side effect (a load that may reach the hardware)."""
    __slots__ = ('t', 'ty', 'uses', 'eff')

    def __init__(self, t, ty, uses=(), eff=False):
        self.t = t
        self.ty = ty
        self.uses = frozenset(uses)
        self.eff = eff

    def __repr__(self):
        return self.t


def var(name, ty):
    return X(name, ty, {name})


def const(v, ty):
    mask = {'u8': 0xFF, 'u16': 0xFFFF, 'u32': 0xFFFFFFFF, 'bool': 1}[ty]
    v &= mask
    if ty == 'bool':
        return X('true' if v else 'false', ty)
    return X('0x%X_%s' % (v, ty) if v > 9 else '%d_%s' % (v, ty), ty)


def mk(t, ty, *parts):
    uses = set()
    eff = False
    for p in parts:
        uses |= p.uses
        eff = eff or p.eff
    return X(t, ty, uses, eff)


def cast(x, ty):
    """x as ty: widening is `from`, narrowing truncates."""
    if x.ty == ty:
        return x
    if ty == 'bool':
        return mk('(%s != 0)' % x.t, ty, x)
    order = ['u8', 'u16', 'u32']
    if x.ty == 'bool':
        return mk('%s::from(%s)' % (ty, x.t), ty, x)
    if order.index(x.ty) < order.index(ty):
        return mk('%s::from(%s)' % (ty, x.t), ty, x)
    return mk('(%s as %s)' % (x.t, ty), ty, x)


def sext(x, ty):
    """x sign-extended to ty."""
    if x.ty == ty:
        return x
    s = {'u8': 'i8', 'u16': 'i16'}[x.ty]
    t = {'u16': 'i16', 'u32': 'i32'}[ty]
    return mk('(%s as %s as %s as %s)' % (x.t, s, t, ty), ty, x)


def binop(op, a, b, ty=None):
    return mk('(%s %s %s)' % (a.t, op, b.t), ty or a.ty, a, b)


def method(a, name, *args, ty=None):
    return mk('%s.%s(%s)' % (a.t, name, ', '.join(x.t for x in args)),
              ty or a.ty, a, *args)


def msb(x):
    return mk('(%s as %s) < 0' % (x.t, {'u8': 'i8', 'u16': 'i16',
                                        'u32': 'i32'}[x.ty]), 'bool', x)


def is_zero(x):
    return mk('%s == 0' % x.t, 'bool', x)


class Lowered:
    """An instruction's statements and its way out."""
    __slots__ = ('stmts', 'term', 'cycles')

    def __init__(self):
        self.stmts = []
        # None: falls through; ('cond', X, target); ('jump', target);
        # ('jumpx', X) an indirect jump; ('call', target); ('callx', X);
        # ('ret',); ('rte',); ('table', X, base) a jump table
        self.term = None


def pure_addr(a):
    """Can a load at the constant address a be dropped when its value is
    unused (work RAM or the ROM, no hardware)?"""
    a &= 0xFFFFFF
    return a >= 0xE00000 or a < 0x400000


class Lowerer:
    def __init__(self, d, depth, frames):
        self.d = d
        self.depth = depth          # {addr: sp's depth before it}
        self.frames = frames        # {addr: {an: depth of its link}}
        self.tmp = 0
        # the type each stack local is used at (arguments: the callee's
        # parameter types)
        self.slot_types = {}
        # stack slots a `move.w sr` saved to (their mask in s.._ipl, the
        # flags in s.._fx and so on)
        self.sr_slots = set()

    # --- values -----------------------------------------------------------

    def temp(self, out, x):
        """x into a fresh temporary (when it is used twice or has an
        effect that must happen once)."""
        self.tmp += 1
        name = 't%d' % self.tmp
        out.stmts.append(('set', name, x))
        return var(name, x.ty)

    def reg(self, n, ty='u32'):
        x = var(REGS[n], 'u32')
        return cast(x, ty) if ty != 'u32' else x

    def set_reg(self, out, n, val, size):
        """A data register written at a size: the rest of it kept."""
        name = REGS[n]
        if size == 'l':
            out.stmts.append(('set', name, cast(val, 'u32')))
            return
        keep = {'b': '0xFFFF_FF00', 'w': '0xFFFF_0000'}[size]
        old = var(name, 'u32')
        out.stmts.append(('set', name, mk('(%s & %s) | u32::from(%s)' % (
            old.t, keep, val.t), 'u32', old, val)))

    def slot(self, pos, size):
        """The local for the stack at pos bytes below the routine's
        entry: a slot of its own (s8), an argument the caller pushed above
        its return address (arg0 the first long above it), or the return
        address itself (ret, which the caller passes: the yield)."""
        if pos <= -4:
            return 'arg%d' % (-pos - 4)
        if pos == 0 and size == 'l':
            return 'ret'
        if pos <= 0:
            raise LowerError('a part of the return address read')
        # by size too: the same place holds a word now and a long later
        # (pushes and pops pair up by size)
        return 's%d_%s' % (pos, size or 'l')

    # --- effective addresses -----------------------------------------------

    def ea(self, out, at, o, size):
        """('mem', X address) or ('slot', name) or ('reg', n) etc. for
        operand o of the instruction at `at`; postincrement and
        predecrement happen here, in order."""
        k = o.kind
        step = BITS[size] // 8 if size else 4
        if k == 'dn':
            return ('d', o.reg)
        if k == 'an':
            if o.reg == 7:
                raise LowerError('sp as a value')
            return ('a', o.reg)
        if k in ('absw', 'absl'):
            # the register gets all 32 bits: a word address sign-extended
            # (0x800A is 0xFFFF800A), a long one as written; the bus
            # sees 24
            v = o.val & 0xFFFF if k == 'absw' else o.val
            if k == 'absw' and v & 0x8000:
                v |= 0xFFFF0000
            return ('mem', const(v, 'u32'), True)
        if k in ('pcdisp', 'pcindex') and k == 'pcdisp':
            return ('mem', const(o.addr, 'u32'), True)
        dep = self.cur_depth
        if o.reg == 7 and k in ('ind', 'disp', 'postinc', 'predec'):
            if k == 'predec':
                n = max(step, 2)
                self.cur_depth += n
                return ('slot', self.slot(self.cur_depth, size))
            if k == 'postinc':
                n = max(step, 2)
                pos = self.cur_depth
                self.cur_depth -= n
                return ('slot', self.slot(pos, size))
            disp = o.val if k == 'disp' else 0
            return ('slot', self.slot(dep - disp, size))
        fr = self.frames.get(at, {})
        if k in ('ind', 'disp') and o.reg in fr:
            disp = o.val if k == 'disp' else 0
            return ('slot', self.slot(fr[o.reg] - disp, size))
        if k == 'ind':
            return ('mem', var(REGS[o.reg + 8], 'u32'), False)
        if k == 'disp':
            a = var(REGS[o.reg + 8], 'u32')
            return ('mem', method(a, 'wrapping_add', const(o.val, 'u32')),
                    False)
        if k == 'postinc':
            if o.reg == 7:
                raise LowerError('(sp)+')
            a = var(REGS[o.reg + 8], 'u32')
            ad = self.temp(out, a)
            out.stmts.append(('set', REGS[o.reg + 8], method(
                a, 'wrapping_add', const(step, 'u32'))))
            return ('mem', ad, False)
        if k == 'predec':
            a = var(REGS[o.reg + 8], 'u32')
            out.stmts.append(('set', REGS[o.reg + 8], method(
                a, 'wrapping_sub', const(step, 'u32'))))
            return ('mem', var(REGS[o.reg + 8], 'u32'), False)
        if k in ('index', 'pcindex'):
            if o.xreg >= 8:
                idx = var(REGS[o.xreg], 'u32') if o.xreg != 15 else None
                if idx is None:
                    raise LowerError('sp as an index')
            else:
                idx = var(REGS[o.xreg], 'u32')
            if o.xsize == 'w':
                idx = sext(cast(idx, 'u16'), 'u32')
            if k == 'index':
                base = method(var(REGS[o.reg + 8], 'u32'), 'wrapping_add',
                              const(o.val, 'u32'))
            else:
                base = const(o.addr, 'u32')
            return ('mem', method(base, 'wrapping_add', idx), False)
        raise LowerError('operand %s' % k)

    def read(self, out, loc, size):
        ty = TY[size]
        if loc[0] == 'd':
            return self.reg(loc[1], ty)
        if loc[0] == 'a':
            return self.reg(loc[1] + 8, ty)
        if loc[0] == 'slot':
            self.slot_types.setdefault(loc[1], ty)
            return var(loc[1], ty)
        if loc[0] == 'imm':
            return const(loc[1], ty)
        _, addr, is_const = loc
        x = mk('self.read%s(%s)' % ({'b': '8', 'w': '16', 'l': '32'}[size],
                                    addr.t), ty, addr)
        if not (is_const and pure_addr(int(addr.t.split('_')[0], 0)
                                       if addr.t[0].isdigit() else 0)):
            x.eff = True
            return self.temp(out, x)
        return x

    def write(self, out, loc, val, size):
        ty = TY[size]
        val = cast(val, ty)
        if loc[0] == 'd':
            self.set_reg(out, loc[1], val, size)
        elif loc[0] == 'a':
            out.stmts.append(('set', REGS[loc[1] + 8], cast(val, 'u32')))
        elif loc[0] == 'slot':
            self.slot_types.setdefault(loc[1], ty)
            out.stmts.append(('set', loc[1], val))
        else:
            out.stmts.append(('store', size, loc[1], val))

    def operand(self, out, at, o, size):
        if o.kind == 'imm':
            return ('imm', o.val)
        if o.kind == 'quick':
            return ('imm', o.val)
        return self.ea(out, at, o, size)

    # --- flags ------------------------------------------------------------

    def flags_logic(self, out, r):
        out.stmts += [('set', 'n', msb(r)), ('set', 'z', is_zero(r)),
                      ('set', 'v', const(0, 'bool')),
                      ('set', 'c', const(0, 'bool'))]

    def flags_add(self, out, a, b, r, x=True, sub=False):
        if sub:
            c = binop('<', a, b, 'bool')
            v = msb(binop('&', binop('^', a, b), binop('^', a, r)))
        else:
            c = binop('<', r, a, 'bool')
            v = msb(binop('&', binop('^', a, r), binop('^', b, r)))
        out.stmts += [('set', 'n', msb(r)), ('set', 'z', is_zero(r)),
                      ('set', 'v', v), ('set', 'c', c)]
        if x:
            out.stmts.append(('set', 'x', var('c', 'bool')))

    # --- conditions -------------------------------------------------------

    def cond(self, cc):
        v = lambda n: var(n, 'bool')
        t = {
            'ra': const(1, 'bool'), 't': const(1, 'bool'),
            'f': const(0, 'bool'),
            'hi': mk('!c && !z', 'bool', v('c'), v('z')),
            'ls': mk('c || z', 'bool', v('c'), v('z')),
            'cc': mk('!c', 'bool', v('c')), 'hs': mk('!c', 'bool', v('c')),
            'cs': v('c'), 'lo': v('c'),
            'ne': mk('!z', 'bool', v('z')), 'eq': v('z'),
            'vc': mk('!v', 'bool', v('v')), 'vs': v('v'),
            'pl': mk('!n', 'bool', v('n')), 'mi': v('n'),
            'ge': mk('n == v', 'bool', v('n'), v('v')),
            'lt': mk('n != v', 'bool', v('n'), v('v')),
            'gt': mk('!z && n == v', 'bool', v('z'), v('n'), v('v')),
            'le': mk('z || n != v', 'bool', v('z'), v('n'), v('v')),
        }
        return t[cc]

    # --- instructions ---------------------------------------------------------

    def lower(self, at):
        i = self.d.insns[at]
        self.cur_depth = self.depth.get(at, 0)
        out = Lowered()
        mn = i.mn
        base, _, size = mn.partition('.')
        size = size or None
        ops = i.ops
        f = getattr(self, 'op_' + base, None)
        if base[0] == 'b' and base[1:] in COND and len(ops) == 1:
            f = self.op_bcc
        elif base.startswith('db') and base[2:] in COND:
            f = self.op_dbcc
        elif base[0] == 's' and base[1:] in COND and base not in (
                'sub', 'subi', 'subq', 'subx', 'swap', 'stop'):
            f = self.op_scc
        if f is None:
            raise LowerError('no lowering for %s' % mn)
        self.cost = timing.cost(i)
        f(out, i, base, size, ops)
        pre, post = self.timing(i, self.cost)
        if self.cost.dyn in ('bcc.s', 'bcc.w'):
            c = out.term[1]
            t = '2 } else { 0' if self.cost.dyn == 'bcc.s' else \
                '0 } else { 2'
            post.append(('eff', mk('self.extra(if %s { %s })' % (c.t, t),
                                   '()', c)))
        out.stmts = pre + out.stmts + post
        out.cycles = timing.cycles(i)
        return out

    @staticmethod
    def timing(i, c):
        """The instruction's time (timing.py, docs/PORT.md "Time"): it
        starts (its base time is spent when the next one does), then
        what it takes before a refresh check (MOVEM's registers)."""
        pre = [('eff', mk('self.insn(0x%06X, %d)' % (i.addr, c.base),
                          '()'))]
        post = []
        if c.pre:
            post.append(('eff', mk('self.extra(%d)' % c.pre, '()')))
        if c.skip and c.dyn is None:
            post.append(('eff', mk('self.skip_refresh()', '()')))
        return pre, post

    def op_nop(self, out, i, base, size, ops):
        pass

    def op_move(self, out, i, base, size, ops):
        src, dst = ops
        if src.kind == 'sr':
            loc = self.ea(out, i.addr, dst, 'w')
            if loc[0] == 'slot':
                # saved on the stack (to be restored): the mask and the
                # flags in locals of their own, so the flags only matter
                # where something after the restore reads them; the slot
                # gets the word without them
                ipl = loc[1] + '_ipl'
                self.slot_types[ipl] = 'u8'
                self.sr_slots.add(loc[1])
                out.stmts.append(('set', ipl, mk('self.ipl()', 'u8')))
                for f in FLAGS:
                    self.slot_types['%s_f%s' % (loc[1], f)] = 'bool'
                    out.stmts.append(('set', '%s_f%s' % (loc[1], f),
                                      var(f, 'bool')))
                self.write(out, loc, mk('(0x2000 | u16::from(%s) << 8)' % ipl,
                                        'u16', var(ipl, 'u8')), 'w')
                return
            self.write(out, loc, mk('self.sr_word(%s)' % ', '.join(FLAGS),
                                    'u16', *[var(n, 'bool') for n in FLAGS]),
                       'w')
            return
        if dst.kind in ('sr', 'ccr'):
            loc = ('none',)
            if src.kind == 'postinc' and src.reg == 7:
                loc = self.ea(out, i.addr, src, 'w')
            if loc[0] == 'slot' and loc[1] in self.sr_slots:
                # an SR saved here: each flag from its own local
                for name in ['c', 'v', 'z', 'n', 'x']:
                    out.stmts.append(('set', name, var('%s_f%s' % (
                        loc[1], name), 'bool')))
            else:
                if loc[0] == 'slot':
                    v = var(loc[1], 'u16')
                else:
                    v = self.read(out, self.operand(out, i.addr, src, 'w'),
                                  'w')
                    v = self.temp(out, v)
                for k, name in enumerate(['c', 'v', 'z', 'n', 'x']):
                    out.stmts.append(('set', name, mk('(%s >> %d) & 1 != 0' % (
                        v.t, k), 'bool', v)))
            if dst.kind == 'sr':
                # the interrupt mask: interrupts wait while it is up
                if loc[0] == 'slot' and loc[1] in self.sr_slots:
                    m = var(loc[1] + '_ipl', 'u8')
                    out.stmts.append(('eff', mk('self.set_ipl(%s)' % m.t,
                                                '()', m)))
                else:
                    out.stmts.append(('eff', mk('self.set_ipl((%s >> 8) as '
                                                'u8 & 7)' % v.t, '()', v)))
            return
        if dst.kind == 'usp':
            return          # the game stays in supervisor mode
        s = self.read(out, self.operand(out, i.addr, src, size), size)
        if s.eff or (src.kind not in ('dn', 'an', 'imm', 'quick') and
                     dst.kind not in ('dn', 'an')):
            s = self.temp(out, s)
        loc = self.ea(out, i.addr, dst, size)
        self.write(out, loc, s, size)
        self.flags_logic(out, s)

    def op_moveq(self, out, i, base, size, ops):
        v = ops[0].val & 0xFF
        v = v - 0x100 if v & 0x80 else v
        s = const(v, 'u32')
        self.write(out, ('d', ops[1].reg), s, 'l')
        self.flags_logic(out, s)

    def op_movea(self, out, i, base, size, ops):
        if ops[0].kind == 'an' and ops[0].reg == 7:
            return              # an alias of the stack (stack.py)
        s = self.read(out, self.operand(out, i.addr, ops[0], size), size)
        if size == 'w':
            s = sext(s, 'u32')
        out.stmts.append(('set', REGS[ops[1].reg + 8], s))

    def op_lea(self, out, i, base, size, ops):
        loc = self.ea(out, i.addr, ops[0], 'l')
        if loc[0] != 'mem':
            raise LowerError('lea of the stack')
        out.stmts.append(('set', REGS[ops[1].reg + 8], loc[1]))

    def op_clr(self, out, i, base, size, ops):
        loc = self.ea(out, i.addr, ops[0], size)
        z = const(0, TY[size])
        self.write(out, loc, z, size)
        self.flags_logic(out, z)

    def op_tst(self, out, i, base, size, ops):
        s = self.read(out, self.ea(out, i.addr, ops[0], size), size)
        self.flags_logic(out, s)

    def rmw(self, out, i, size, src, dst, fn):
        """dst = fn(dst, src), flags from fn."""
        s = self.read(out, self.operand(out, i.addr, src, size), size) \
            if src is not None else None
        if s is not None and s.eff:
            s = self.temp(out, s)
        loc = self.ea(out, i.addr, dst, size)
        if loc[0] == 'mem' and loc[1].uses:
            loc = ('mem', self.temp(out, loc[1]), loc[2])
        dv = self.read(out, loc, size)
        dv = self.temp(out, dv) if not dv.t.isidentifier() else dv
        r = self.temp(out, fn(dv, s))
        self.write(out, loc, r, size)
        return dv, s, r

    def op_add(self, out, i, base, size, ops):
        if ops[-1].kind == 'an':
            return self.op_adda(out, i, base, size, ops)
        a, b, r = self.rmw(out, i, size, ops[0], ops[1],
                           lambda d, s: method(d, 'wrapping_add',
                                               cast(s, d.ty)))
        self.flags_add(out, a, cast(b, a.ty), r)
    op_addi = op_add
    op_addq = op_add

    def op_sub(self, out, i, base, size, ops):
        if ops[-1].kind == 'an':
            return self.op_suba(out, i, base, size, ops)
        a, b, r = self.rmw(out, i, size, ops[0], ops[1],
                           lambda d, s: method(d, 'wrapping_sub',
                                               cast(s, d.ty)))
        self.flags_add(out, a, cast(b, a.ty), r, sub=True)
    op_subi = op_sub
    op_subq = op_sub

    def op_adda(self, out, i, base, size, ops):
        n = ops[1].reg
        if n == 7 or n in self.frames.get(i.addr, {}):
            return              # sp or an alias of it: the stack pass
        s = self.read(out, self.operand(out, i.addr, ops[0], size), size)
        if size == 'w':
            s = sext(s, 'u32')
        out.stmts.append(('set', REGS[n + 8], method(
            var(REGS[n + 8], 'u32'), 'wrapping_add', cast(s, 'u32'))))

    def op_suba(self, out, i, base, size, ops):
        n = ops[1].reg
        if n == 7:
            return
        s = self.read(out, self.operand(out, i.addr, ops[0], size), size)
        if size == 'w':
            s = sext(s, 'u32')
        out.stmts.append(('set', REGS[n + 8], method(
            var(REGS[n + 8], 'u32'), 'wrapping_sub', cast(s, 'u32'))))

    def op_addx(self, out, i, base, size, ops):
        ty = TY[size]

        def f(d, s, sub=False):
            return mk('%s.wrapping_%s(%s).wrapping_%s(%s::from(x))' % (
                d.t, 'sub' if sub else 'add', cast(s, ty).t,
                'sub' if sub else 'add', ty), ty, d, s, var('x', 'bool'))
        a, b, r = self.rmw(out, i, size, ops[0], ops[1], f)
        b = cast(b, ty)
        out.stmts += [
            ('set', 'c', mk('u64::from(%s) + u64::from(%s) + u64::from(x) '
                            '> %s::MAX as u64' % (a.t, b.t, ty), 'bool', a, b,
                            var('x', 'bool'))),
            ('set', 'v', msb(binop('&', binop('^', a, r), binop('^', b, r)))),
            ('set', 'n', msb(r)),
            ('set', 'z', mk('z && %s == 0' % r.t, 'bool', var('z', 'bool'),
                            r)),
            ('set', 'x', var('c', 'bool'))]

    def op_subx(self, out, i, base, size, ops):
        ty = TY[size]
        a, b, r = self.rmw(out, i, size, ops[0], ops[1], lambda d, s: mk(
            '%s.wrapping_sub(%s).wrapping_sub(%s::from(x))' % (
                d.t, cast(s, ty).t, ty), ty, d, s, var('x', 'bool')))
        b = cast(b, ty)
        out.stmts += [
            ('set', 'c', mk('u64::from(%s) < u64::from(%s) + u64::from(x)' %
                            (a.t, b.t), 'bool', a, b, var('x', 'bool'))),
            ('set', 'v', msb(binop('&', binop('^', a, b), binop('^', a, r)))),
            ('set', 'n', msb(r)),
            ('set', 'z', mk('z && %s == 0' % r.t, 'bool', var('z', 'bool'),
                            r)),
            ('set', 'x', var('c', 'bool'))]

    def op_cmp(self, out, i, base, size, ops):
        s = self.read(out, self.operand(out, i.addr, ops[0], size), size)
        if s.eff:
            s = self.temp(out, s)
        dv = self.read(out, self.ea(out, i.addr, ops[1], size), size)
        if ops[1].kind == 'an':
            dv = self.reg(ops[1].reg + 8, TY[size])
        if dv.eff:
            dv = self.temp(out, dv)
        s = cast(s, dv.ty)
        r = method(dv, 'wrapping_sub', s)
        out.stmts += [('set', 'n', msb(r)), ('set', 'z', mk(
            '%s == %s' % (dv.t, s.t), 'bool', dv, s)),
            ('set', 'v', msb(binop('&', binop('^', dv, s),
                                   binop('^', dv, r)))),
            ('set', 'c', binop('<', dv, s, 'bool'))]
    op_cmpi = op_cmp

    def logic(self, out, i, size, ops, op):
        if ops[-1].kind in ('ccr', 'sr'):
            v = ops[0].val
            if ops[-1].kind == 'sr':
                m = (v >> 8) & 7
                if op == '&':
                    e = 'self.set_ipl(self.ipl() & %d)' % m
                elif op == '|':
                    e = 'self.set_ipl(self.ipl() | %d)' % m
                else:
                    e = 'self.set_ipl(self.ipl() ^ %d)' % m
                if m or op == '&':
                    out.stmts.append(('eff', X(e, '()')))
            for k, name in enumerate(['c', 'v', 'z', 'n', 'x']):
                bit = v >> k & 1
                cur = var(name, 'bool')
                if op == '&' and not bit:
                    out.stmts.append(('set', name, const(0, 'bool')))
                elif op == '|' and bit:
                    out.stmts.append(('set', name, const(1, 'bool')))
                elif op == '^' and bit:
                    out.stmts.append(('set', name, mk('!%s' % name, 'bool',
                                                      cur)))
            return
        a, b, r = self.rmw(out, i, size, ops[0], ops[1],
                           lambda d, s: binop(op, d, cast(s, d.ty)))
        self.flags_logic(out, r)

    def op_and(self, out, i, base, size, ops):
        self.logic(out, i, size, ops, '&')
    op_andi = op_and

    def op_or(self, out, i, base, size, ops):
        self.logic(out, i, size, ops, '|')
    op_ori = op_or

    def op_eor(self, out, i, base, size, ops):
        self.logic(out, i, size, ops, '^')
    op_eori = op_eor

    def op_not(self, out, i, base, size, ops):
        a, b, r = self.rmw(out, i, size, None, ops[0],
                           lambda d, s: mk('!%s' % d.t, d.ty, d))
        self.flags_logic(out, r)

    def op_neg(self, out, i, base, size, ops):
        a, b, r = self.rmw(out, i, size, None, ops[0],
                           lambda d, s: method(d, 'wrapping_neg'))
        z = const(0, a.ty)
        self.flags_add(out, z, a, r, sub=True)

    def op_ext(self, out, i, base, size, ops):
        n = ops[0].reg
        if size == 'w':
            r = sext(self.reg(n, 'u8'), 'u16')
        else:
            r = sext(self.reg(n, 'u16'), 'u32')
        r = self.temp(out, r)
        self.set_reg(out, n, r, size)
        self.flags_logic(out, r)

    def op_swap(self, out, i, base, size, ops):
        n = ops[0].reg
        r = method(var(REGS[n], 'u32'), 'rotate_left', const(16, 'u32'))
        out.stmts.append(('set', REGS[n], r))
        self.flags_logic(out, var(REGS[n], 'u32'))

    def op_exg(self, out, i, base, size, ops):
        a = REGS[ops[0].reg + (8 if ops[0].kind == 'an' else 0)]
        b = REGS[ops[1].reg + (8 if ops[1].kind == 'an' else 0)]
        t = self.temp(out, var(a, 'u32'))
        out.stmts += [('set', a, var(b, 'u32')), ('set', b, t)]

    def shift(self, out, i, base, size, ops, kind):
        if len(ops) == 1:           # memory, by one
            cnt = const(1, 'u32')
            dst = ops[0]
            size = 'w'
        else:
            dst = ops[1]
            if ops[0].kind in ('quick', 'imm'):
                cnt = const(ops[0].val or 8, 'u32')
            else:
                cnt = mk('(%s & 63)' % REGS[ops[0].reg], 'u32',
                         var(REGS[ops[0].reg], 'u32'))
                # 2 cycles a bit (timing.py: dyn 'shift')
                out.stmts.append(('eff', mk('self.shift_time(%s)' % cnt.t,
                                            '()', cnt)))
        ty = TY[size]
        fn = 'self.%s%d' % (kind, BITS[size])
        xin = [] if kind in ('asl', 'asr', 'lsl', 'lsr', 'rol', 'ror') \
            else ['x']
        a, b, r = self.rmw(out, i, size, None, dst, lambda d, s: d)
        # rmw wrote d back unchanged; the real work: a call that returns
        # the value and the flags
        out.stmts.pop()
        res = self.temp(out, mk('%s(%s, %s%s)' % (
            fn, a.t, cnt.t, ''.join(', ' + v for v in xin)),
            '(%s, bool, bool, bool)' % ty, a, cnt,
            *[var(v, 'bool') for v in xin]))
        rv = mk('%s.0' % res.t, ty, res)
        self.write(out, self.last_loc, rv, size) if False else None
        out.stmts.append(('setloc', None, rv))
        out.stmts += [('set', 'c', mk('%s.1' % res.t, 'bool', res)),
                      ('set', 'v', mk('%s.2' % res.t, 'bool', res)),
                      ('set', 'n', msb(rv)), ('set', 'z', is_zero(rv))]
        if kind in ('roxl', 'roxr') and not cnt.uses:
            # by a count in the instruction: the bit rotated out (C and
            # X) comes from the operand alone, never the X rotated in
            k = cnt.t.split('_')[0]
            k = int(k, 0)
            bit = BITS[size] - k if kind == 'roxl' else k - 1
            out_bit = mk('(%s >> %d) & 1 != 0' % (a.t, bit), 'bool', a)
            out.stmts[-4] = ('set', 'c', out_bit)
            out.stmts.append(('set', 'x', out_bit))
        elif kind not in ('rol', 'ror'):
            if not cnt.uses:
                # a count in the instruction: never 0, X is set
                out.stmts.append(('set', 'x', mk('%s.3' % res.t, 'bool',
                                                 res)))
            else:
                out.stmts.append(('set', 'x', mk(
                    'if %s != 0 { %s.3 } else { x }' % (cnt.t, res.t),
                    'bool', cnt, res, var('x', 'bool'))))
        return dst, size, rv

    def op_shift(self, out, i, base, size, ops):
        dst, sz, rv = self.shift(out, i, base, size, ops, base)
        # the result back where it came from
        loc = self.ea(out, i.addr, dst, sz) if dst.kind in ('dn',) else None
        out.stmts = [s for s in out.stmts if s[0] != 'setloc']
        if loc is None:
            if dst.kind not in ('absw', 'absl'):
                raise LowerError('shift of memory through a register')
            loc = ('mem', const(dst.addr, 'u32'), True)
        self.write(out, loc, rv, sz)
    op_asl = op_asr = op_lsl = op_lsr = op_rol = op_ror = op_roxl = \
        op_roxr = op_shift

    def bitop(self, out, i, base, size, ops):
        if ops[0].kind == 'imm':
            bit = const(ops[0].val & (31 if size == 'l' else 7), 'u32')
        else:
            n = ops[0].reg
            bit = mk('(%s & %d)' % (REGS[n], 31 if size == 'l' else 7),
                     'u32', var(REGS[n], 'u32'))
        ty = TY[size]
        if self.cost.dyn == 'bit':
            # a bit above 15 takes 2 more
            out.stmts.append(('eff', mk('self.extra(if %s >= 16 { 2 } else '
                                        '{ 0 })' % bit.t, '()', bit)))
        loc = self.ea(out, i.addr, ops[1], size)
        if loc[0] == 'mem' and loc[1].uses:
            loc = ('mem', self.temp(out, loc[1]), loc[2])
        dv = self.read(out, loc, size)
        if dv.eff or not dv.t.isidentifier():
            dv = self.temp(out, dv)
        m = mk('(1 << %s)' % bit.t, ty, bit) if ty == 'u32' else \
            mk('(1_%s << %s)' % (ty, bit.t), ty, bit)
        out.stmts.append(('set', 'z', mk('%s & %s == 0' % (dv.t, m.t),
                                         'bool', dv, m)))
        if base == 'btst':
            return
        op = {'bset': '%s | %s', 'bclr': '%s & !%s', 'bchg': '%s ^ %s'}[base]
        self.write(out, loc, mk('(' + op % (dv.t, m.t) + ')', ty, dv, m),
                   size)

    op_btst = op_bset = op_bclr = op_bchg = bitop

    def op_mulu(self, out, i, base, size, ops):
        s = self.temp(out, self.read(out, self.operand(out, i.addr, ops[0],
                                                       'w'), 'w'))
        out.stmts.append(('eff', mk('self.mulu_time(%s)' % s.t, '()', s)))
        n = ops[1].reg
        r = self.temp(out, mk('u32::from(%s) * u32::from(%s)' % (
            self.reg(n, 'u16').t, s.t), 'u32', var(REGS[n], 'u32'), s))
        out.stmts.append(('set', REGS[n], r))
        self.flags_logic(out, r)

    def op_divu(self, out, i, base, size, ops):
        s = self.temp(out, self.read(out, self.operand(out, i.addr, ops[0],
                                                       'w'), 'w'))
        n = ops[1].reg
        out.stmts.append(('eff', mk('self.divu_time(%s, %s)' % (
            REGS[n], s.t), '()', var(REGS[n], 'u32'), s)))
        r = self.temp(out, mk('self.divu(%s, %s)' % (REGS[n], s.t),
                              '(u32, bool)', var(REGS[n], 'u32'), s))
        out.stmts += [('set', REGS[n], mk('%s.0' % r.t, 'u32', r)),
                      ('set', 'v', mk('%s.1' % r.t, 'bool', r)),
                      ('set', 'c', const(0, 'bool')),
                      ('set', 'n', mk('(%s.0 as u16 as i16) < 0' % r.t,
                                      'bool', r)),
                      ('set', 'z', mk('%s.0 as u16 == 0' % r.t, 'bool', r))]

    def op_movem(self, out, i, base, size, ops):
        src, dst = ops
        step = 4 if size == 'l' else 2
        if src.kind == 'reglist':
            mask = src.val
            if dst.kind == 'predec':
                mask = int('{:016b}'.format(mask)[::-1], 2)
            regs = [k for k in range(16) if mask >> k & 1]
            if dst.kind == 'predec' and dst.reg == 7:
                # stored highest register first, so the lowest ends at sp
                for k in reversed(regs):
                    self.cur_depth += step
                    if k == 15:
                        continue        # sp itself: its value is unused
                    name = self.slot(self.cur_depth, size)
                    self.slot_types.setdefault(name, TY[size])
                    out.stmts.append(('set', name, self.reg(k, TY[size])))
                return
            raise LowerError('movem to memory')
        regs = [k for k in range(16) if dst.val >> k & 1]
        if src.kind == 'postinc' and src.reg == 7:
            for k in regs:
                name = self.slot(self.cur_depth, size)
                self.slot_types.setdefault(name, TY[size])
                v = var(name, TY[size])
                self.cur_depth -= step
                if k == 15:
                    continue
                if size == 'w':
                    v = sext(v, 'u32')
                out.stmts.append(('set', REGS[k], v))
            return
        loc = self.ea(out, i.addr, src, 'l')
        if loc[0] != 'mem':
            raise LowerError('movem from %s' % loc[0])
        base_addr = self.temp(out, loc[1])
        for n, k in enumerate(regs):
            v = mk('self.read%d(%s.wrapping_add(%d))' % (
                BITS[size], base_addr.t, n * step), TY[size], base_addr)
            v.eff = True
            if size == 'w':
                v = sext(v, 'u32')
            out.stmts.append(('set', REGS[k], v))
        if src.kind == 'postinc':
            # (an)+: an moves past what was read (ea() moved it by one
            # long), over a value loaded into it
            out.stmts.append(('set', REGS[src.reg + 8], mk(
                '%s.wrapping_add(%d)' % (base_addr.t, len(regs) * step),
                'u32', base_addr)))

    def op_link(self, out, i, base, size, ops):
        # an = the frame (stack.py tracks it as an alias)
        self.cur_depth += 4
        name = self.slot(self.cur_depth, 'l')
        self.slot_types.setdefault(name, 'u32')
        out.stmts.append(('set', name, self.reg(ops[0].reg + 8)))

    def op_unlk(self, out, i, base, size, ops):
        n = ops[0].reg
        frame = self.frames[i.addr][n]
        out.stmts.append(('set', REGS[n + 8], var(self.slot(frame, 'l'),
                                                  'u32')))

    # --- control ----------------------------------------------------------

    def op_bcc(self, out, i, base, size, ops):
        cc = base[1:]
        if cc == 'ra':
            out.term = ('jump', i.target)
        elif cc == 'sr':
            out.term = ('call', i.target)
        else:
            out.term = ('cond', self.cond(cc), i.target)

    def op_dbcc(self, out, i, base, size, ops):
        cc = base[2:]
        n = ops[0].reg
        c = self.cond(cc)
        cnt = self.temp(out, mk('%s.wrapping_sub(1)' % self.reg(n, 'u16').t,
                                'u16', var(REGS[n], 'u32')))
        # the counter is decremented only while the condition is false
        if cc in ('ra', 'f'):
            self.set_reg(out, n, cnt, 'w')
            # looping 10 cycles, the count run out 14
            out.stmts.append(('eff', mk('self.extra(if %s == 0xFFFF { 4 } '
                                        'else { 0 })' % cnt.t, '()', cnt)))
            out.term = ('cond', mk('%s != 0xFFFF' % cnt.t, 'bool', cnt),
                        i.target)
        else:
            stay = self.temp(out, mk('!(%s)' % c.t, 'bool', c))
            # the condition true 12, the count run out 14
            out.stmts.append(('eff', mk(
                'self.extra(if !%s { 2 } else if %s == 0xFFFF { 4 } else '
                '{ 0 })' % (stay.t, cnt.t), '()', stay, cnt)))
            self.set_reg(out, n, mk('if %s { %s } else { %s }' % (
                stay.t, cnt.t, self.reg(n, 'u16').t), 'u16', stay, cnt,
                var(REGS[n], 'u32')), 'w')
            out.term = ('cond', mk('%s && %s != 0xFFFF' % (stay.t, cnt.t),
                                   'bool', stay, cnt), i.target)

    def op_scc(self, out, i, base, size, ops):
        c = self.cond(base[1:])
        if self.cost.dyn == 'scc':
            out.stmts.append(('eff', mk('self.extra(if %s { 2 } else { 0 })'
                                        % c.t, '()', c)))
        loc = self.ea(out, i.addr, ops[0], 'b')
        self.write(out, loc, mk('if %s { 0xFF } else { 0 }' % c.t, 'u8', c),
                   'b')

    def op_jmp(self, out, i, base, size, ops):
        o = ops[0]
        if o.kind in ('absw', 'absl'):
            out.term = ('jump', o.addr)
        elif o.kind == 'pcindex':
            out.term = ('table', var(REGS[o.xreg], 'u32'), o.addr)
        else:
            loc = self.ea(out, i.addr, o, 'l')
            out.term = ('jumpx', loc[1])

    def op_jsr(self, out, i, base, size, ops):
        o = ops[0]
        if o.kind in ('absw', 'absl'):
            out.term = ('call', o.addr)
        elif o.kind == 'pcindex':
            out.term = ('calltable', var(REGS[o.xreg], 'u32'), o.addr)
        else:
            loc = self.ea(out, i.addr, o, 'l')
            out.term = ('callx', loc[1])

    def op_rts(self, out, i, base, size, ops):
        out.term = ('ret',)

    def op_rte(self, out, i, base, size, ops):
        out.term = ('rte',)


COND = ['ra', 'sr', 't', 'f', 'hi', 'ls', 'cc', 'cs', 'hs', 'lo', 'ne', 'eq',
        'vc', 'vs', 'pl', 'mi', 'ge', 'lt', 'gt', 'le']
