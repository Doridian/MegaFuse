"""Which routines each call through a register or a table can reach
(tools/recomp).

The traces say where each indirect `jsr`/`jmp` went (calls.VERSION.txt,
mdtrace --edges).  Sites that share a target form a family; its members
share one signature (dataflow.py), so a call through the family's
pointer can reach any of them.  Members no trace reached join a family
from the code:

- a table the code calls through (`movea.l (tbl,pc,dn),an; jsr (an)`,
  `jmp (tbl,pc,dn)`): its entries, with its site's family;
- any other routine whose address the code stores (`move.l #f,...`)
  goes with the family most traced routines stored the same way belong
  to: pushed as an argument to func_001222, func_00053E or func_0003E8
  (a task's routine), written to (0x2,an) or 0xFF80D0 (an object's), to
  0xFF80E0 (the scene)...

What is left over (no trace, no telling store) forms one family of its
own.
"""
import bisect
import collections
import os

TOP = os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.abspath(__file__))))


def load_calls(version):
    path = os.path.join(TOP, 'calls.%s.txt' % version)
    out = []
    if os.path.exists(path):
        for line in open(path):
            t = line.split('#')[0].split()
            if t:
                out.append((int(t[0], 16), int(t[1], 16)))
    return out


def store_patterns(d, ks, t):
    """How the code stores the address t: 'arg F' (pushed, then a call
    to F), or 'mnemonic dest-kind where'."""
    pats = set()
    for a, i in d.insns.items():
        for o in i.ops:
            if o.kind == 'imm' and o.size == 'l' and o.val == t:
                dst = i.ops[-1]
                if dst.kind == 'predec' and dst.reg == 7:
                    j = bisect.bisect_left(ks, a)
                    for k in ks[j + 1:j + 5]:
                        n = d.insns[k]
                        if n.mn == 'jsr' and n.target is not None:
                            pats.add('arg %X' % n.target)
                            break
                elif dst.kind == 'disp':
                    pats.add('%s (%d,an)' % (i.mn, dst.val))
                elif dst.kind in ('absw', 'absl'):
                    pats.add('%s %X' % (i.mn, dst.addr))
    return pats


def families(d, routines, version, taken):
    """[(sites, members)]: taken, the routines whose address the code or
    a pointer names (they may be called through a pointer)."""
    ks = sorted(d.insns)
    calls = [(s, t) for s, t in load_calls(version) if t in routines]
    parent = {}

    def find(x):
        while parent.setdefault(x, x) != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x
    sites = set(s for s, _ in calls)
    # the tables the code calls through: their entries
    table_of = {}
    for a, i in d.insns.items():
        if i.mn in ('jsr', 'jmp') and i.ops[0].kind == 'pcindex':
            sites.add(a)
            table_of[a] = jump_table(d, i.ops[0].addr, routines)
        elif i.mn in ('jsr', 'jmp') and i.ops[0].kind in ('ind', 'index',
                                                          'disp'):
            sites.add(a)
    for s in sites:
        find(s)
    by_target = collections.defaultdict(set)
    for s, t in calls:
        by_target[t].add(s)
    for s, ts in table_of.items():
        for t in ts:
            by_target[t].add(s)
    for t, ss in by_target.items():
        ss = sorted(ss)
        for s in ss[1:]:
            parent[find(s)] = find(ss[0])
    members = collections.defaultdict(set)
    for t, ss in by_target.items():
        members[find(next(iter(ss)))].add(t)
    # the rest of what is taken, by how it is stored
    vote = collections.defaultdict(collections.Counter)
    for root, ms in members.items():
        for t in ms:
            for p in store_patterns(d, ks, t):
                vote[p][root] += 1
    known = set().union(*members.values()) if members else set()
    left = set()
    for t in sorted(taken - known):
        best = collections.Counter()
        for p in store_patterns(d, ks, t):
            best.update(vote.get(p, {}))
        if best:
            members[best.most_common(1)[0][0]].add(t)
        else:
            left.add(t)
    out = []
    groups = collections.defaultdict(set)
    for s in sites:
        groups[find(s)].add(s)
    for root, ss in groups.items():
        out.append((ss, members.get(root, set())))
    if left:
        out.append((set(), left))
    return out


def jump_table(d, base, routines):
    """The routines a table at base calls or jumps to: longs (a call
    table) or 4-byte entries of code (a jump table, its entries are
    routines themselves)."""
    out = set()
    if base in d.jtbl:
        a = base
        while a in d.jtbl_entries:
            if a in routines:
                out.add(a)
            a += 4
        return out
    for a, v in d.call_table(base):
        if v in routines:
            out.add(v)
    return out
