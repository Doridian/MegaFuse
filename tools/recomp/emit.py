"""Rust for a routine (tools/recomp).

A routine becomes a method of `Game` (crates/game):

- an ordinary routine `fn r_XXXXXX(&mut self, <arguments>) -> <results>`:
  its arguments the registers and flags it reads (dataflow.py's IN), the
  stack arguments its callers push (argN) and the return address when it
  reads it (ret, the yield); its results what its callers read
  (dataflow.py's OUT), and `bool` last when it can return from its
  caller (stack.py's unwind);
- a routine called through a pointer (families.py) `fn
  r_XXXXXX(&mut self, regs: &mut Regs)`: the registers its family shares;
- the vectors' routines take and give nothing; the exception handlers
  panic.

Registers, flags and stack slots are locals.  What nothing reads is not
written (a statement-level liveness pass), so most flags never appear.
"""
from recomp import dataflow
from recomp.lower import REGS, FLAGS

ORDER = REGS + FLAGS
RTY = {r: 'u32' for r in REGS}
RTY.update({f: 'bool' for f in FLAGS})


def fname(e):
    return 'r_%06x' % e


def sorted_res(xs):
    return [x for x in ORDER if x in xs]


class Emitter:
    def __init__(self, prog, flow, roots):
        self.p = prog
        self.f = flow
        self.roots = roots

    # --- signatures ----------------------------------------------------

    def is_family(self, e):
        return e in self.f.fam_of and e not in self.roots

    def params(self, r):
        """[(name, type)] of an ordinary routine."""
        out = [(x, RTY[x]) for x in sorted_res(self.f.IN[r.entry])]
        for n in sorted(r.args):
            out.append(('arg%d' % n, r.slot_types.get('arg%d' % n, 'u32')))
        if r.reads_ret:
            out.append(('ret', 'u32'))
        return out

    def results(self, r):
        out = [(x, RTY[x]) for x in sorted_res(self.f.OUT[r.entry])]
        if r.unwind is not False:
            out.append(('unwound', 'bool'))
        return out

    def ret_type(self, res):
        if not res:
            return ''
        if len(res) == 1:
            return ' -> %s' % res[0][1]
        return ' -> (%s)' % ', '.join(t for _, t in res)

    # --- liveness over every local ------------------------------------------

    def call_io(self, op):
        """(uses, defs) of a call statement over all locals."""
        if op[0] == 'call':
            callee = self.p.routines.get(op[1])
            u, d = dataflow.op_uses_defs(op, self.f.sig)
            if callee is not None:
                for n in callee.args:
                    pos = op[2] - n
                    if pos > 0:
                        sv = self.slot_value(pos, callee.slot_types.get(
                            'arg%d' % n, 'u32'))
                        if sv is not None:
                            u.add(sv[0])
                    else:
                        u.add(self.slot_name(pos))
            if callee is not None and self.is_family(op[1]):
                fk = self.f.fam_of[op[1]]
                u |= set(self.f.fam_in[fk])
                d = set(self.f.fam_out[fk])
            return u, d
        u, d = dataflow.op_uses_defs(op, self.f.sig)
        u |= set(op[1].uses)
        return u, d

    def slot_value(self, pos, ty):
        """(the caller's local holding the n bytes at pos, the expression
        for them as ty): a slot of its own (s<position>_<size>, position
        bytes below the entry, covering pos down to pos - bytes + 1) that
        contains them, read big-endian (the DMA routine's length is a word
        its callers push and it reads as two bytes), or None."""
        n = {'u8': 1, 'u16': 2, 'u32': 4}.get(ty, 4)
        best = None
        for name, st in self.r.slot_types.items():
            if not dataflow.is_slot(name):
                continue
            P = int(name[1:].partition('_')[0])
            S = {'u8': 1, 'u16': 2, 'u32': 4}[st]
            if st == 'u8':
                S = 1
            if P >= pos and pos - n + 1 >= P - S + 1:
                if best is None or S < best[2]:
                    best = (name, st, S, P)
        if best is None:
            return None
        name, st, S, P = best
        if S == n and P == pos:
            return name, name
        shift = 8 * (S - (P - pos) - n)
        return name, '((%s >> %d) as %s)' % (name, shift, ty) if shift else \
            '(%s as %s)' % (name, ty)

    def slot_name(self, pos, size=None):
        """The caller's local at pos: a slot of its own (the size the
        callee reads it at, else whichever size the caller wrote it at),
        its own argument, or its return address."""
        if pos <= -4:
            return 'arg%d' % (-pos - 4)
        if pos > 0:
            names = ['s%d_%s' % (pos, z) for z in
                     ([size] if size else []) + ['l', 'w', 'b']]
            for n in names:
                if n in self.r.slot_types:
                    return n
            return names[0]
        return 'ret'

    def out_set(self, r):
        e = r.entry
        if self.is_family(e):
            return set(self.f.fam_out[self.f.fam_of[e]])
        return set(self.f.OUT[e])

    def term_uses(self, r, start):
        b = r.blocks[start]
        t = r.term.get(start)
        out = set()
        if t is None:
            if b.kind == 'exit':
                last = b.insns[-1]
                nxt = last + self.p.d.insns[last].size
                out |= self.callee_in(nxt) | self.out_set(r)
            return out
        k = t[0]
        if k == 'cond':
            out |= set(t[1].uses)
            if t[2] not in r.blocks:
                out |= self.callee_in(t[2]) | self.out_set(r)
        elif k == 'jump' and t[1] not in r.blocks:
            out |= self.callee_in(t[1]) | self.out_set(r)
        elif k in ('jumpx', 'table'):
            i, _ = self.f.sig(('site', b.insns[-1]))
            out |= set(t[1].uses) | set(i) | self.out_set(r)
        elif k == 'ret':
            out |= self.out_set(r)
        if r.unwind is not False:
            out.add('unw')
        return out

    def callee_in(self, e):
        r = self.p.routines.get(e)
        if r is None:
            return set(dataflow.RES)
        if self.is_family(e):
            return set(self.f.fam_in[self.f.fam_of[e]])
        return set(self.f.IN[e]) | set('arg%d' % n for n in r.args)

    def liveness(self, r):
        """{(block, index): live after statement index} over all
        locals."""
        live_at = {s: set() for s in r.blocks}
        after = {}
        changed = True
        out_r = self.out_set(r)
        while changed:
            changed = False
            for start in sorted(r.blocks, reverse=True):
                b = r.blocks[start]
                live = set(self.term_uses(r, start))
                for s2, _ in b.succs:
                    live |= live_at[s2]
                ops = r.ops[start]
                for k in range(len(ops) - 1, -1, -1):
                    after[(start, k)] = set(live)
                    op = ops[k]
                    if op[0] == 'set':
                        live.discard(op[1])
                        live |= set(op[2].uses)
                    elif op[0] == 'store':
                        live |= set(op[2].uses) | set(op[3].uses)
                    elif op[0] == 'eff':
                        live |= set(op[1].uses)
                    else:
                        u, df = self.call_io(op)
                        unw = self.p.routines.get(op[1]) \
                            if op[0] == 'call' else None
                        if unw is not None and unw.unwind is not False:
                            live |= out_r
                        live = (live - df) | u
                if live != live_at[start]:
                    live_at[start] = live
                    changed = True
        return after

    # --- the routine ----------------------------------------------------

    def routine(self, r):
        e = r.entry
        if r.crash:
            return ('    pub fn %s(&mut self) {\n        panic!("exception: '
                    'the game crashed (its handler at {:#08X})", 0x%X_u32);\n'
                    '    }\n' % (fname(e), e))
        if r.error:
            return ('    pub fn %s(&mut self, _regs: &mut Regs) {\n'
                    '        unimplemented!("routine %06X: %s");\n    }\n' % (
                        fname(e), e, r.error.replace('"', "'")))
        self.r = r
        self.waits = set()
        self.live = self.liveness(r)
        self.family = self.is_family(e)
        self.root = e in self.roots
        lines = []
        if self.root:
            sig = 'pub fn %s(&mut self)' % fname(e)
            params = []
        elif self.family:
            sig = 'pub fn %s(&mut self, regs: &mut Regs)' % fname(e)
            params = []
        else:
            params = self.params(r)
            sig = 'pub fn %s(&mut self%s)%s' % (
                fname(e), ''.join(', %s: %s' % (n, t) for n, t in params),
                self.ret_type(self.results(r)))
        lines.append('    #[allow(unused_mut, unused_variables, '
                     'unused_assignments, unreachable_code, '
                     'clippy::all)]')
        lines.append('    %s {' % sig)
        body = []
        pnames = set(n for n, _ in params)
        for x in ORDER:
            if x in pnames:
                body.append('let mut %s: %s = %s;' % (x, RTY[x], x))
            elif self.family and x in self.f.fam_in[self.f.fam_of[e]]:
                body.append('let mut %s: %s = regs.%s;' % (x, RTY[x], x))
            else:
                body.append('let mut %s: %s = %s;' % (
                    x, RTY[x], '0' if RTY[x] == 'u32' else 'false'))
        body.append('let mut unw: bool = false;')
        for name, ty in sorted(r.slot_types.items()):
            if dataflow.is_slot(name):
                body.append('let mut %s: %s = 0;' % (name, ty))
        body += self.node(r.tree, 0)
        lines += ['        ' + l for l in body]
        lines.append('    }')
        return '\n'.join(lines) + '\n'

    def ret_stmt(self, unwound=False):
        """Return what the routine gives back."""
        r = self.r
        if self.root:
            return ['return;']
        if self.family:
            outs = sorted_res(self.f.fam_out[self.f.fam_of[r.entry]])
            return ['regs.%s = %s;' % (x, x) for x in outs] + [
                'regs.unwound = %s;' % ('true' if unwound else 'unw'),
                'return;']
        res = self.results(r)
        vals = [n for n, _ in res]
        if r.unwind is not False:
            vals[-1] = 'true' if unwound else 'unw'
        if not vals:
            return ['return;']
        if len(vals) == 1:
            return ['return %s;' % vals[0]]
        return ['return (%s);' % ', '.join(vals)]

    def node(self, n, depth):
        k = n[0]
        if k == 'seq':
            out = []
            for c in n[1]:
                out += self.node(c, depth)
            return out
        if k == 'code':
            return self.code(n[1])
        if k == 'if':
            t = self.r.term[n[1]]
            cond = t[1].t
            return (['if %s {' % cond] + ind(self.node(n[2], depth + 1)) +
                    ['} else {'] + ind(self.node(n[3], depth + 1)) + ['}'])
        if k == 'labeled':
            return (['%s: {' % n[1]] + ind(self.node(n[2], depth + 1)) +
                    ['}'])
        if k == 'loop':
            wait = self.polls(n[2])
            if wait:
                self.waits.add(n[1])
            body = self.node(n[2], depth + 1)
            self.waits.discard(n[1])
            head = ['// waits for %s' % wait] if wait else []
            return head + ['%s: loop {' % n[1]] + ind(body) + ['}']
        if k == 'break':
            return ['break %s;' % n[1]]
        if k == 'continue':
            return ['continue %s;' % n[1]]
        if k == 'tail':
            return self.tail_call(n[1])
        if k == 'exit':
            return self.exit(n[1])
        raise ValueError(k)

    def blocks_in(self, n, out):
        if n[0] in ('code', 'exit'):
            out.add(n[1])
        elif n[0] == 'if':
            out.add(n[1])
            self.blocks_in(n[2], out)
            self.blocks_in(n[3], out)
        elif n[0] == 'seq':
            for c in n[1]:
                self.blocks_in(c, out)
        elif n[0] in ('labeled', 'loop'):
            self.blocks_in(n[2], out)
        return out

    def polls(self, n):
        """A loop that only reads work RAM (at fixed addresses) and
        branches: it waits for an interrupt handler to change what it
        reads.  Returns what it reads, or None."""
        reads = set()
        for start in self.blocks_in(n, set()):
            for op in self.r.ops[start]:
                if op[0] != 'set':
                    return None
                t = op[2].t
                if 'self.read' in t:
                    import re
                    for m in re.finditer(r'self\.read\d+\(0x([0-9A-F_]+)_u32\)', t):
                        a = int(m.group(1).replace('_', ''), 16)
                        if a < 0xE00000:
                            return None
                        reads.add(a)
                    if 'self.read' in re.sub(
                            r'self\.read\d+\(0x[0-9A-F_]+_u32\)', '', t):
                        return None
            term = self.r.term.get(start)
            b = self.r.blocks[start]
            if term is None:
                if b.kind == 'exit':
                    return None          # falls into another routine
            elif term[0] in ('ret', 'rte'):
                pass
            elif term[0] == 'cond' and term[2] in self.r.blocks:
                pass
            elif term[0] == 'jump' and term[1] in self.r.blocks:
                pass
            else:
                return None
        if not reads:
            return None
        return ', '.join('0x%06X' % (a & 0xFFFFFF) for a in sorted(reads))

    def code(self, start):
        # the block's time: the beam moves on, interrupts may come
        # (docs/PORT.md, "Time")
        out = ['self.tick(%d);' % self.r.cycles.get(start, 4)]
        for k, op in enumerate(self.r.ops[start]):
            live = self.live.get((start, k), set())
            if op[0] == 'set':
                name, x = op[1], op[2]
                if name not in live and not x.eff:
                    continue
                if name.startswith('t') and name[1:].isdigit():
                    out.append('let %s: %s = %s;' % (name, x.ty, x.t))
                else:
                    out.append('%s = %s;' % (name, x.t))
            elif op[0] == 'store':
                size, a, v = op[1], op[2], op[3]
                out.append('self.write%s(%s, %s);' % (
                    {'b': '8', 'w': '16', 'l': '32'}[size], a.t, v.t))
            elif op[0] == 'eff':
                out.append('%s;' % op[1].t)
            else:
                out += self.call(op, live)
        return out

    # --- calls ----------------------------------------------------------------

    def regs_in(self):
        return 'Regs { %s, unwound: false }' % ', '.join(ORDER)

    def regs_back(self, outs):
        return ['%s = regs.%s;' % (x, x) for x in outs]

    def call_direct(self, e, depth, ret):
        """Statements calling routine e, its results into the locals, and
        whether it may return for us (the name of that flag, or None)."""
        callee = self.p.routines.get(e)
        if callee is None:
            return ['unimplemented!("a call into the middle of code at '
                    '{:#08X}", 0x%X_u32);' % e], None
        if callee.crash:
            return ['self.%s();' % fname(e)], None
        if e in self.roots:
            return ['self.%s();' % fname(e)], None
        if self.is_family(e):
            outs = sorted_res(self.f.fam_out[self.f.fam_of[e]])
            return (['{', '    let mut regs = %s;' % self.regs_in(),
                     '    self.%s(&mut regs);' % fname(e)] +
                    ['    ' + l for l in self.regs_back(outs)] +
                    ['    unwound = regs.unwound;', '}'],
                    'unwound' if callee.unwind is not False else None)
        args = []
        for n, t in self.params(callee):
            if n == 'ret':
                args.append('0x%X_u32' % ret)
            elif n.startswith('arg'):
                pos = depth - int(n[3:])
                if pos > 0:
                    sv = self.slot_value(pos, t)
                    args.append(sv[1] if sv else '0')
                else:
                    slot = self.slot_name(pos)
                    have = self.r.slot_types.get(slot)
                    if have is None:
                        args.append('0')
                    elif have != t:
                        args.append('%s as %s' % (slot, t))
                    else:
                        args.append(slot)
            else:
                args.append(n)
        res = self.results(callee)
        call = 'self.%s(%s)' % (fname(e), ', '.join(args))
        names = [n for n, _ in res]
        if not names:
            return [call + ';'], None
        if len(names) == 1:
            return ['%s = %s;' % (names[0], call)], \
                'unwound' if names[0] == 'unwound' else None
        return ['(%s) = %s;' % (', '.join(names), call)], \
            'unwound' if names[-1] == 'unwound' else None

    def call_family(self, site, target):
        k = self.f.fam_of_site.get(site)
        outs = sorted_res(self.f.fam_out[k]) if k is not None else ORDER
        return (['{', '    let mut regs = %s;' % self.regs_in(),
                 '    self.call_family_%s(%s, &mut regs);' % (
                     k if k is not None else 'any', target)] +
                ['    ' + l for l in self.regs_back(outs)] +
                ['    unwound = regs.unwound;', '}'])

    def call(self, op, live):
        if op[0] == 'call':
            stmts, unw = self.call_direct(op[1], op[2], op[3])
            out = ['let mut unwound = false;'] + stmts \
                if unw or any('unwound' in s for s in stmts) else stmts
            if unw:
                out += ['if unwound {'] + ind(self.ret_stmt()) + ['}']
            return out
        if op[0] == 'callx':
            target = op[1].t
        else:
            target = '0x%X_u32.wrapping_add(%s as u16 as i16 as i32 as u32)' \
                % (op[2], op[1].t)
        out = ['let mut unwound = false;'] + self.call_family(op[-1], target)
        out += ['if unwound {'] + ind(self.ret_stmt()) + ['}']
        return out

    def tail_call(self, e):
        stmts, unw = self.call_direct(e, 0, 0)
        out = ['let mut unwound = false;'] + stmts
        out += self.ret_stmt()
        return ['{'] + ind(out) + ['}']

    def exit(self, start):
        r = self.r
        b = r.blocks[start]
        t = r.term.get(start)
        last = b.insns[-1]
        if t is None:
            nxt = last + self.p.d.insns[last].size
            return self.tail_call(nxt)
        k = t[0]
        if k == 'ret':
            dep = r.depth.get(last, 0)
            return self.ret_stmt(unwound=False)
        if k == 'rte':
            return ['return;']
        if k == 'jump':
            return self.tail_call(t[1])
        if k == 'cond':
            return self.tail_call(t[2])
        if k == 'jumpx':
            out = ['let mut unwound = false;'] + self.call_family(
                last, t[1].t) + self.ret_stmt()
            return ['{'] + ind(out) + ['}']
        if k == 'table':
            target = '0x%X_u32.wrapping_add(%s as u16 as i16 as i32 as u32)' \
                % (t[2], t[1].t)
            out = ['let mut unwound = false;'] + self.call_family(
                last, target) + self.ret_stmt()
            return ['{'] + ind(out) + ['}']
        return ['unimplemented!("%s");' % (k,)]


def size_of(ty):
    return {'u8': 'b', 'u16': 'w', 'u32': 'l'}.get(ty)


def ind(lines):
    return ['    ' + l for l in lines]
