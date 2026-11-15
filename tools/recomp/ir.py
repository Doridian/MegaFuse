"""The routines as statements in structured control flow (tools/recomp).

build(d) puts the passes together: routines.py's routines, stack.py's
depths, structure.py's tree and lower.py's statements, and returns a
Program: for each routine its blocks' statements, calls among them, and
how each block leaves.

A call is a statement ('call', callee, depth, ret): callee the routine's
entry (or an X for a call through a register: ('callx', X, depth, ret)),
depth sp's depth before the `jsr` (which slots are the callee's
arguments), ret the return address (what a routine that reads its
return address, the yield, gets).
"""
import collections

from recomp import timing, lower, routines, stack, structure

# the exception handlers (vectors 2-23, the traps' 0x254) and their
# stubs: crash screens, a panic in the port.  0x260 (the unused
# interrupts: it waits forever) and 0x266 (`rte`) are ordinary, and
# 0x268 on is the game's code
CRASH_RANGE = (0x200, 0x260)


class Routine:
    __slots__ = ('entry', 'body', 'blocks', 'tree', 'ops', 'term',
                 'depth', 'crash', 'unwind', 'reads_ret', 'args', 'error',
                 'slot_types', 'cycles', 'preserved')

    def __init__(self, entry, body):
        self.entry = entry
        self.body = body
        self.blocks = {}
        self.tree = None
        self.ops = {}          # block start -> [statements]
        self.term = {}         # block start -> its way out (lower.Lowered.term)
        self.depth = {}
        self.crash = False
        # may return from its caller too: the bytes of the caller's frame
        # it pops (the caller must be that deep at the call), else False
        self.unwind = False
        self.reads_ret = False
        self.args = set()      # argN slots it reads
        self.error = None
        self.slot_types = {}   # stack local -> its Rust type
        self.cycles = {}       # block start -> its 68000 cycles
        # registers it saves and restores (dataflow.drop_saves): its
        # callers' are untouched whatever it does with them
        self.preserved = set()


class Program:
    def __init__(self, d):
        self.d = d
        self.routines = {}
        self.inline = set()


def build(d):
    p = Program(d)
    rs, p.inline = routines.partition_balanced(d)
    for e, body in sorted(rs.items()):
        r = Routine(e, body)
        p.routines[e] = r
        if CRASH_RANGE[0] <= e < CRASH_RANGE[1]:
            r.crash = True
            continue
        try:
            r.depth, alias = stack.depths(d, body)
            r.blocks, r.tree = structure.structure(d, body)
            low = lower.Lowerer(d, r.depth, alias)
            for start, b in r.blocks.items():
                ops = []
                term = None
                r.cycles[start] = 0
                for k, a in enumerate(b.insns):
                    i = d.insns[a]
                    if a == 0x7C4A and i.mn == 'movem.l':
                        # us: sp reloaded before the main loop; its time
                        pre, post = lower.Lowerer.timing(
                            i, timing.cost(i))
                        ops += pre + post
                        continue
                    lw = low.lower(a)
                    ops += lw.stmts
                    r.cycles[start] += lw.cycles
                    # popping the routine's own return address: it returns
                    # from its caller too (the local `unw`, emit.py)
                    if i.mn != 'unlk' and i.flow != 'ret':
                        before = r.depth[a]
                        after = stack.effect(i, before)
                        if before > -4 >= after:
                            ops.append(('set', 'unw', lower.const(1, 'bool')))
                    t = lw.term
                    last = k == len(b.insns) - 1
                    if t is not None and t[0] in ('call', 'callx',
                                                  'calltable') and not last:
                        ops.append(call_op(t, r.depth[a], a + i.size, a))
                        t = None
                    if last:
                        # a jump to the main loop's head (the end of the
                        # boot, a pass of the loop) returns: the step
                        if t is not None and t[0] == 'jump' and \
                                t[1] == routines.STEP:
                            t = ('ret',)
                        term = t
                        if t is not None and t[0] in ('call', 'callx',
                                                      'calltable'):
                            ops.append(call_op(t, r.depth[a], a + i.size,
                                               a))
                            term = None
                r.ops[start] = ops
                r.term[start] = term
            r.slot_types = low.slot_types
            u = stack.UNWINDS.get(e, set())
            if len(u) > 1:
                raise stack.StackError('returns from the caller with %s '
                                       'bytes of its frame' % sorted(u))
            r.unwind = next(iter(u)) if u else False
            for ops in r.ops.values():
                for op in ops:
                    if op[0] == 'set':
                        for u in op[2].uses:
                            note_arg(r, u)
                        note_arg(r, op[1], write=True)
                    elif op[0] == 'store':
                        for u in op[2].uses | op[3].uses:
                            note_arg(r, u)
        except (stack.StackError, lower.LowerError,
                structure.Irreducible) as x:
            r.error = str(x)
    return p


def note_arg(r, name, write=False):
    if name == 'ret':
        r.reads_ret = True
    elif name.startswith('arg'):
        r.args.add(int(name[3:]))


def call_op(t, depth, ret, site):
    """('call', callee, depth, ret, site), ('callx', X, depth, ret, site),
    ('calltable', X, table, depth, ret, site): site the `jsr`'s address
    (which family a call through a pointer reaches, families.py)."""
    if t[0] == 'call':
        return ('call', t[1], depth, ret, site)
    if t[0] == 'callx':
        return ('callx', t[1], depth, ret, site)
    return ('calltable', t[1], t[2], depth, ret, site)


def summary(p):
    c = collections.Counter()
    for r in p.routines.values():
        c['crash' if r.crash else 'error' if r.error else 'ok'] += 1
        if r.unwind:
            c['unwind'] += 1
        if r.reads_ret:
            c['reads ret'] += 1
        if r.args:
            c['takes stack args'] += 1
    return c
