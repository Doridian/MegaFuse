"""Which registers and flags each routine takes and gives back
(tools/recomp).

The registers (d0-d7, a0-a6) and the flags (x n z v c) are values that
routines pass each other: a routine's arguments are those it reads
before writing on some path (given what it hands back), its results
those it may change that a caller reads after the call.  Worked out over
the whole program, to a fixpoint:

  MOD(r)  what r or its callees may write
  OUT(r)  MOD(r) that some caller reads after calling r (or, for a tail
          call, that the routine tail-calling it hands back)
  IN(r)   what is live at r's entry when OUT(r) is live at its returns

Routines called through a pointer (task routines, jump table entries,
code the traces found entered indirectly, continuations) share one
signature, so a call through a pointer can go to any of them: IN and OUT
the unions over them.  Interrupt handlers and the reset routine take
nothing and give nothing back.
"""
from recomp.lower import REGS, FLAGS

RES = frozenset(REGS + FLAGS)


def op_uses_defs(op, sig):
    """(uses, defs) of a statement, sig(callee or ('site', a)) -> (IN,
    OUT)."""
    k = op[0]
    if k == 'set':
        return set(op[2].uses) & RES, {op[1]} & RES
    if k == 'store':
        return set(op[2].uses | op[3].uses) & RES, set()
    if k == 'eff':
        return set(op[1].uses) & RES, set()
    if k == 'call':
        i, o = sig(op[1])
        return set(i), set(o)
    if k in ('callx', 'calltable'):
        i, o = sig(('site', op[-1]))
        x = op[1]
        return set(i) | (set(x.uses) & RES), set(o)
    raise ValueError(k)


class Flow:
    def __init__(self, prog, fams):
        """fams: [(sites, members)] (families.py)."""
        self.p = prog
        self.fams = [(set(s), set(m) & set(prog.routines)) for s, m in fams]
        self.fam_of_site = {}
        self.fam_of = {}
        for k, (ss, ms) in enumerate(self.fams):
            for s in ss:
                self.fam_of_site[s] = k
            for m in ms:
                self.fam_of.setdefault(m, k)
        self.indirect = set(self.fam_of)
        self.IN = {e: set() for e in prog.routines}
        self.OUT = {e: set() for e in prog.routines}
        self.MOD = {e: set() for e in prog.routines}
        self.fam_in = [set() for _ in self.fams]
        self.fam_out = [set() for _ in self.fams]
        self.roots = set()                # take and give nothing

    def sig(self, e):
        if isinstance(e, tuple):           # a call through a pointer
            k = self.fam_of_site.get(e[1])
            if k is None:
                return set(RES), set(RES)
            return self.fam_in[k], self.fam_out[k]
        if e is None or e not in self.p.routines:
            return set(RES), set(RES)
        return self.IN[e], self.OUT[e]

    def fam_members(self, site):
        k = self.fam_of_site.get(site)
        return self.fams[k][1] if k is not None else set()

    # --- MOD ----------------------------------------------------------

    def compute_mod(self):
        changed = True
        while changed:
            changed = False
            for e, r in self.p.routines.items():
                m = set()
                for ops in r.ops.values():
                    for op in ops:
                        if op[0] == 'set':
                            m |= {op[1]} & RES
                        elif op[0] == 'call':
                            m |= self.MOD.get(op[1], set())
                        elif op[0] in ('callx', 'calltable'):
                            for t in self.fam_members(op[-1]):
                                m |= self.MOD[t]
                for t in self.tails(r):
                    if isinstance(t, tuple):
                        for x in self.fam_members(t[1]):
                            m |= self.MOD[x]
                    else:
                        m |= self.MOD.get(t, set())
                m -= r.preserved
                if not m <= self.MOD[e]:
                    self.MOD[e] |= m
                    changed = True

    def tails(self, r):
        """Routines r tail-calls (None: one through a pointer)."""
        out = []
        for start, b in r.blocks.items():
            t = r.term.get(start)
            if t is None:
                continue
            if t[0] == 'jump' and t[1] not in r.blocks:
                out.append(t[1])
            elif t[0] == 'cond' and t[2] not in r.blocks:
                out.append(t[2])
            elif t[0] in ('jumpx', 'table'):
                out.append(('site', b.insns[-1]))
        # a fall-through into another routine
        for start, b in r.blocks.items():
            if b.kind == 'exit' and r.term.get(start) is None:
                last = b.insns[-1]
                nxt = last + self.p.d.insns[last].size
                if nxt in self.p.routines:
                    out.append(nxt)
        return out

    # --- liveness -------------------------------------------------------

    def live_in(self, r, need):
        """Liveness over r's blocks with OUT(r) live at its returns:
        (live at the entry, {call site index: live after the call},
        what r's tail calls hand back that must be live)."""
        d = self.p.d
        live_at = {s: set() for s in r.blocks}
        after_call = {}
        changed = True
        while changed:
            changed = False
            for start in sorted(r.blocks, reverse=True):
                b = r.blocks[start]
                live = self.block_out(r, start, b, live_at)
                ops = r.ops[start]
                for k in range(len(ops) - 1, -1, -1):
                    op = ops[k]
                    if op[0] in ('call', 'callx', 'calltable'):
                        callee = op[1] if op[0] == 'call' else \
                            ('site', op[-1])
                        prev = after_call.get((start, k), set())
                        a = set(live)
                        unw = self.p.routines.get(callee) \
                            if not isinstance(callee, tuple) else None
                        if unw is not None and unw.unwind is not False:
                            a |= self.OUT[r.entry]   # it may return for r
                        if a != prev:
                            after_call[(start, k)] = a
                    u, df = op_uses_defs(op, self.sig)
                    if op[0] == 'call':
                        unw = self.p.routines.get(op[1])
                        if unw is not None and unw.unwind is not False:
                            live |= self.OUT[r.entry]
                    live = (live - df) | u
                if live != live_at[start]:
                    live_at[start] = live
                    changed = True
        return live_at[r.entry], after_call

    def block_out(self, r, start, b, live_at):
        """What is live where block b leaves."""
        t = r.term.get(start)
        out = set()
        for s, _ in b.succs:
            out |= live_at[s]
        if t is None:
            if b.kind == 'exit':          # falls into another routine
                last = b.insns[-1]
                nxt = last + self.p.d.insns[last].size
                i, _ = self.sig(nxt)
                out |= set(i) | self.OUT[r.entry]
            return out
        k = t[0]
        if k == 'cond':
            out |= set(t[1].uses)
            if t[2] not in r.blocks:          # a tail call
                i, _ = self.sig(t[2])
                out |= set(i) | self.OUT[r.entry]
        elif k == 'jump' and t[1] not in r.blocks:
            i, _ = self.sig(t[1])
            out |= set(i) | self.OUT[r.entry]
        elif k in ('jumpx', 'table'):
            i, _ = self.sig(('site', b.insns[-1]))
            out |= set(t[1].uses) | set(i) | self.OUT[r.entry]
        elif k == 'ret':
            out |= self.OUT[r.entry]
        return out & RES

    # --- the fixpoint ----------------------------------------------------

    def solve(self, roots):
        self.roots = set(roots)
        self.compute_mod()
        fam_mod = [set().union(*(self.MOD[m] for m in ms)) if ms else set()
                   for _, ms in self.fams]
        changed = True
        rounds = 0
        while changed:
            changed = False
            rounds += 1
            needed = {e: set() for e in self.p.routines}
            needed_fam = [set() for _ in self.fams]
            for e, r in self.p.routines.items():
                if r.error or r.crash:
                    continue
                ins, after = self.live_in(r, None)
                if e not in self.roots and not ins <= self.IN[e]:
                    self.IN[e] |= ins & RES
                    changed = True
                for (start, k), live in after.items():
                    op = r.ops[start][k]
                    if op[0] == 'call':
                        needed.setdefault(op[1], set())
                        needed[op[1]] |= live
                    else:
                        f = self.fam_of_site.get(op[-1])
                        if f is not None:
                            needed_fam[f] |= live
                for t in self.tails(r):
                    if isinstance(t, tuple):
                        f = self.fam_of_site.get(t[1])
                        if f is not None:
                            needed_fam[f] |= self.OUT[e]
                    else:
                        needed.setdefault(t, set())
                        needed[t] |= self.OUT[e]
            for e in self.p.routines:
                if e in self.roots:
                    continue
                o = needed.get(e, set()) & self.MOD[e]
                f = self.fam_of.get(e)
                if f is not None:
                    o |= needed_fam[f] & fam_mod[f]
                if not o <= self.OUT[e]:
                    self.OUT[e] |= o
                    changed = True
            for f, (_, ms) in enumerate(self.fams):
                fi = set().union(*(self.IN[m] for m in ms)) if ms else set()
                fo = needed_fam[f] & fam_mod[f]
                if fi != self.fam_in[f] or fo != self.fam_out[f]:
                    self.fam_in[f], self.fam_out[f] = fi, fo
                    changed = True
            # a family member takes what any member takes (one signature)
            for f, (_, ms) in enumerate(self.fams):
                for m in ms:
                    if m not in self.roots and \
                            not self.fam_in[f] <= self.IN[m]:
                        self.IN[m] |= self.fam_in[f]
                        changed = True
        return rounds


def is_slot(name):
    """A stack local (lower.py's s<position>_<size>)?"""
    if not name.startswith('s') or '_' not in name:
        return False
    pos, _, size = name[1:].partition('_')
    return pos.isdigit() and size in ('b', 'w', 'l')


def drop_saves(r, prog=None):
    """Registers r saves and restores (`movem` to the stack and back): in
    Rust they are the caller's locals anyway, so the save and the restore
    go and the register isn't something r reads or changes for its
    callers.  A forward pass tags what still holds a register's value
    from the entry (copies through stack slots keep the tag; any other
    write, and any call, loses it); a register whose own tag reaches every
    way out is preserved, and its save and restore statements, when the
    slot is read by nothing else, are dropped.  Returns the preserved
    registers."""
    from recomp.lower import REGS as R
    regs = set(R)
    starts = sorted(r.blocks)
    tags_in = {r.entry: {x: x for x in regs}}
    # in reverse postorder-ish: blocks in address order, repeated
    out_of = {}
    exits = []
    for _ in range(3):
        for s in starts:
            if s not in tags_in:
                continue
            t = dict(tags_in[s])
            for op in r.ops[s]:
                if op[0] == 'set':
                    src = op[2]
                    if src.t in t or (src.t.isidentifier() and src.t
                                      in t):
                        t[op[1]] = t.get(src.t)
                    elif is_slot(src.t):
                        t[op[1]] = t.get(src.t)
                    else:
                        t.pop(op[1], None)
                elif op[0] in ('call', 'callx', 'calltable'):
                    for x in regs:
                        t.pop(x, None)
            out_of[s] = t
            for nxt, _ in r.blocks[s].succs:
                if nxt in tags_in:
                    old = tags_in[nxt]
                    tags_in[nxt] = {k: v for k, v in old.items()
                                    if t.get(k) == v}
                else:
                    tags_in[nxt] = dict(t)
    for s, b in r.blocks.items():
        term = r.term.get(s)
        if s not in out_of:
            continue
        if term is not None and term[0] in ('ret', 'rte'):
            exits.append(out_of[s])
        elif b.kind == 'exit' or term is not None and (
                term[0] in ('jumpx', 'table') or
                (term[0] == 'jump' and term[1] not in r.blocks) or
                (term[0] == 'cond' and term[2] not in r.blocks)):
            # a tail call: what it calls may change any register and hand
            # it back, so nothing is preserved through it
            exits.append({})
    if not exits:
        return set()
    preserved = set(x for x in regs if all(e.get(x) == x for e in exits))
    # the statements: saves (slot = X while X holds its own value) and
    # restores (X = slot holding X's value)
    saves, restores, reads = {}, {}, {}
    for s in starts:
        for k, op in enumerate(r.ops.get(s, [])):
            if op[0] != 'set':
                continue
            src = op[2].t
            if is_slot(op[1]) and src in preserved:
                saves[(s, k)] = (op[1], src)
            elif op[1] in preserved and is_slot(src):
                restores[(s, k)] = (op[1], src)
            for u in op[2].uses:
                if is_slot(u):
                    reads.setdefault(u, []).append((s, k))
        # what a callee reads above its return address: the caller's
        # slots (func_010BA2 pops what its callers saved)
        for k, op in enumerate(r.ops.get(s, [])):
            if op[0] == 'call' and prog is not None:
                callee = prog.routines.get(op[1])
                for n in (callee.args if callee else ()):
                    pos = op[2] - n
                    if pos > 0:
                        for sz in 'bwl':
                            reads.setdefault('s%d_%s' % (pos, sz), []).append(
                                (s, ('call', k)))
    drop = set()
    for key, (slot, x) in saves.items():
        users = reads.get(slot, [])
        if all(u in restores and restores[u] == (x, slot) for u in users):
            drop.add(key)
            drop |= set(users)
    for s in starts:
        r.ops[s] = [op for k, op in enumerate(r.ops.get(s, []))
                    if (s, k) not in drop]
    return preserved
