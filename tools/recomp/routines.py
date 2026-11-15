"""The code cut into routines, the units tools/recomp turns into Rust
functions (docs/PORT.md, "Translation").

A routine is an entry and everything reached from it without passing
another entry: falling through or jumping into another routine is a tail
call, a `jsr`/`bsr` a call (the routine goes on after it).  The entries are
the disassembly's functions, the code entries the traces found, the jump
table entries and code that pointers name, and then, until no instruction
belongs to two routines, every shared instruction that one of them reaches
from outside the other (a shared tail becomes a routine of its own).
"""
import collections

JUMP_TABLE_STEP = 4


def successors(d, a):
    """Where control goes after the instruction at a, within the code:
    (address, kind) with kind 'next' (falls through, or a call returns),
    'branch' (a branch or jump to a fixed target).  A call's target is not
    a successor: it is another routine."""
    i = d.insns[a]
    out = []
    nxt = a + i.size
    if i.flow in (None, 'call'):
        out.append((nxt, 'next'))
    elif i.flow == 'branch':
        out.append((nxt, 'next'))
        if i.target is not None:
            out.append((i.target, 'branch'))
    elif i.flow == 'jump':
        if i.target is not None:
            out.append((i.target, 'branch'))
    return [(t, k) for t, k in out if t in d.insns]


def entries_of(d):
    out = set(a for a in d.funcs if a in d.insns)
    out |= set(a for a in d.extra if a in d.insns)
    out |= set(a for a in d.jtbl_entries if a in d.insns)
    out |= set(t for t in d.ptrs.values() if t in d.insns)
    for i in d.insns.values():
        for t, kind in i.refs():
            if kind == 'imm' and t in d.insns:
                out.add(t)       # `move.l #code,...`: a code pointer
    return out


def region(d, starts, entries):
    """What is reached from starts without passing an entry."""
    seen = set(starts)
    todo = list(starts)
    while todo:
        a = todo.pop()
        for s, _ in successors(d, a):
            if s not in entries and s not in seen:
                seen.add(s)
                todo.append(s)
    return seen


def partition(d, inline=()):
    """{entry: sorted instruction addresses} covering the code.  inline:
    starts of shared tails that are copied into each routine reaching
    them rather than made routines (an epilogue that restores registers
    saved before the jump into it)."""
    entries = entries_of(d) - set(inline)
    while True:
        owner = collections.defaultdict(set)
        for e in entries:
            seen = {e}
            todo = [e]
            while todo:
                a = todo.pop()
                for s, _ in successors(d, a):
                    if s in entries or s in seen:
                        continue
                    seen.add(s)
                    todo.append(s)
            for a in seen:
                owner[a].add(e)
        preds = collections.defaultdict(set)
        for a in owner:
            for s, _ in successors(d, a):
                preds[s].add(a)
        copied = region(d, inline, entries)
        more = set()
        for a, es in owner.items():
            if a in copied:
                continue
            if len(es) > 1 and any(owner[p] != es for p in preds[a]):
                more.add(a)
        if not more:
            break
        entries |= more
    out = collections.defaultdict(list)
    for a, es in owner.items():
        assert len(es) == 1 or a in copied, hex(a)
        for e in es:
            out[e].append(a)
    lost = [a for a in d.insns if a not in owner]
    # the entry first, then the rest in address order
    return {e: [e] + sorted(a for a in v if a != e)
            for e, v in out.items()}, lost


def partition_balanced(d):
    """partition(), the shared tails stack.py finds reached with
    something still on the stack copied instead of split: (routines,
    the copied tails)."""
    from recomp import stack
    inline = set()
    while True:
        rs, lost = partition(d, inline)
        more = set()
        for e, body in rs.items():
            try:
                stack.depths(d, body)
            except stack.StackError as x:
                if x.tail is not None:
                    more.add(x.tail)
        if not more - inline:
            return rs, inline
        inline |= more
