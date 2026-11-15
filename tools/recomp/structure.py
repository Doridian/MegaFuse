"""A routine's control flow as nested blocks and loops (tools/recomp).

The method of Ramsey's "Beyond Relooper" (ICFP 2022), as WebAssembly
compilers use it, which fits Rust's labeled blocks and loops: over the
dominator tree of the routine's basic blocks, in reverse postorder,

- a block with two or more forward edges into it (a merge) gets a labeled
  block that ends right before it, opened in its immediate dominator:
  `break 'label` goes there;
- a block a backward edge goes to (a loop header) gets a labeled `loop`:
  `continue 'label` goes back there;
- any other block is emitted inline where its one predecessor branches
  to it.

The result is a tree of nodes:

  ('seq', [nodes])
  ('code', block)                  the block's instructions but its last
                                   (when that one is a branch or jump)
  ('if', block, then, else)        the block's conditional branch
  ('labeled', label, node)         a block: break label leaves it
  ('loop', label, node)            continue label goes back to its start
  ('break', label) ('continue', label)
  ('exit', block)                  the block's last instruction leaves the
                                   routine (return, tail call, jump table)
  ('tail', address)                a conditional branch's way out: a tail
                                   call to the routine at address

Graphs where a loop has two entries (irreducible) are reported.
"""
from recomp.routines import successors


class Irreducible(Exception):
    pass


class Block:
    __slots__ = ('start', 'insns', 'succs', 'kind')

    def __init__(self, start):
        self.start = start
        self.insns = []
        self.succs = []       # [(block start, 'next' | 'branch')]
        self.kind = None      # 'fall', 'cond', 'jump', 'exit'

    def __repr__(self):
        return 'B%06X' % self.start


def basic_blocks(d, body):
    inside = set(body)
    leaders = {body[0]}
    for a in body:
        i = d.insns[a]
        succ = successors(d, a)
        if i.flow in ('branch', 'jump', 'ret', 'stop') or \
                any(k == 'branch' for _, k in succ):
            for s, _ in succ:
                if s in inside:
                    leaders.add(s)
            nxt = a + i.size
            if nxt in inside:
                leaders.add(nxt)
    blocks = {}
    for a in body:
        if a in leaders:
            cur = blocks[a] = Block(a)
        cur.insns.append(a)
    for b in blocks.values():
        last = b.insns[-1]
        i = d.insns[last]
        succ = [(s, k) for s, k in successors(d, last) if s in inside]
        out = [(s, k) for s, k in succ]
        if i.flow == 'branch':
            b.kind = 'cond'
        elif i.flow == 'jump' and out:
            b.kind = 'jump'
        elif i.flow in ('jump', 'ret', 'stop') or not out or \
                any(s not in inside for s, _ in successors(d, last)):
            b.kind = 'exit' if not out else b.kind
        if b.kind is None:
            b.kind = 'fall'
        b.succs = out
    return blocks


def structure(d, body):
    blocks = basic_blocks(d, body)
    entry = body[0]
    # reverse postorder
    order, seen = [], set()

    def dfs(a):
        stack = [(a, iter([s for s, _ in blocks[a].succs]))]
        seen.add(a)
        while stack:
            n, it = stack[-1]
            for s in it:
                if s not in seen:
                    seen.add(s)
                    stack.append((s, iter([t for t, _ in blocks[s].succs])))
                    break
            else:
                order.append(n)
                stack.pop()
    dfs(entry)
    rpo = list(reversed(order))
    index = {a: k for k, a in enumerate(rpo)}
    preds = {a: [] for a in rpo}
    for a in rpo:
        for s, _ in blocks[a].succs:
            preds[s].append(a)
    # dominators (Cooper, Harvey, Kennedy)
    idom = {entry: entry}
    changed = True
    while changed:
        changed = False
        for a in rpo[1:]:
            ps = [p for p in preds[a] if p in idom]
            new = ps[0]
            for p in ps[1:]:
                x, y = p, new
                while x != y:
                    while index[x] > index[y]:
                        x = idom[x]
                    while index[y] > index[x]:
                        y = idom[y]
                new = x
            if idom.get(a) != new:
                idom[a] = new
                changed = True

    def dominates(x, y):
        while True:
            if x == y:
                return True
            if y == entry:
                return False
            y = idom[y]

    loops = set()
    for a in rpo:
        for s, _ in blocks[a].succs:
            if index[s] <= index[a]:
                if not dominates(s, a):
                    raise Irreducible('%06X -> %06X' % (a, s))
                loops.add(s)
    merges = set(a for a in rpo
                 if sum(1 for p in preds[a] if index[p] < index[a]) >= 2)
    children = {a: [] for a in rpo}
    for a in rpo[1:]:
        children[idom[a]].append(a)

    def do_tree(x, ctx):
        ms = sorted((c for c in children[x] if c in merges),
                    key=lambda c: index[c], reverse=True)
        if x in loops:
            return ('loop', label(x), node_within(x, ms, ctx + [('loop', x)]))
        return node_within(x, ms, ctx)

    def node_within(x, ys, ctx):
        if not ys:
            b = blocks[x]
            parts = [('code', x)]
            if b.kind == 'cond':
                i = d.insns[b.insns[-1]]
                taken = arm(x, i.target, ctx)
                fall = arm(x, i.addr + i.size, ctx)
                parts = [('code', x), ('if', x, taken, fall)]
            elif b.kind in ('fall', 'jump'):
                parts.append(do_branch(x, b.succs[0][0], ctx))
            else:
                parts.append(('exit', x))
            return ('seq', parts)
        y, rest = ys[0], ys[1:]
        inner = node_within(x, rest, ctx + [('block', y)])
        return ('seq', [('labeled', label(y), inner), do_tree(y, ctx)])

    def arm(src, t, ctx):
        """A conditional branch's way: within the routine, or a tail
        call to another routine."""
        if t in blocks:
            return do_branch(src, t, ctx)
        return ('tail', t)

    def do_branch(src, t, ctx):
        if index[t] <= index[src]:
            return ('continue', label(t))
        if t in merges:
            return ('break', label(t))
        return do_tree(t, ctx)

    return blocks, do_tree(entry, [])


def label(a):
    return "'l%06x" % a
