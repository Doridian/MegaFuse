#!/usr/bin/env python3
"""What the runtime traces never ran, sorted out: docs/UNRUN.md.

    tools/unrun.py [VERSION]          (us by default; reads build/V/trace/)

Run after tools/trace_all.sh.  Every instruction of the disassembly that
no trace ran belongs to a function (the nearest function label at or
before it), and each such function falls in one of these groups:

  dead      nothing that could run names it: no reference at all (the
            dead siblings tools/disasm.py's gap filling finds), or only
            from functions that are dead themselves
  never     named by code that ran, or by a table: live code no script
            has triggered (the ones to hunt down)
  partly    the function ran, some of its instructions didn't (branches
            not taken)

A reference is a call, a branch or jump, an address the code names
(`lea`, `move.l #x`), or a pointer in a table of the data between code.
A pointer from a table counts as live: tables are read by indexes the
traces may not have reached.
"""
import bisect
import collections
import glob
import os
import sys

TOP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(TOP, 'tools'))
import disasm  # noqa: E402

OUT = os.path.join(TOP, 'docs', 'UNRUN.md')


def main():
    v = sys.argv[1] if len(sys.argv) > 1 else 'us'
    d = disasm.analyze(v)
    lim = d.code_limit
    cov = bytearray(lim)
    paths = glob.glob(os.path.join(TOP, 'build', v, 'trace', '*.cov'))
    if not paths:
        sys.exit('no traces in build/%s/trace (tools/trace_all.sh %s)' %
                 (v, v))
    per = {}                    # script -> its coverage
    for p in sorted(paths):
        c = open(p, 'rb').read()[:lim]
        per[os.path.basename(p)[:-4]] = c
        for i in range(lim):
            cov[i] |= c[i]
    funcs = sorted(a for a in d.funcs if a in d.insns)

    def owner(a):
        return funcs[bisect.bisect_right(funcs, a) - 1]

    ran = collections.Counter()
    size = collections.Counter()
    for a in d.insns:
        f = owner(a)
        size[f] += 1
        if cov[a] & 1:
            ran[f] += 1
    # who names each function: (referrer function or None for a table,
    # did the referring instruction run)
    named = collections.defaultdict(list)
    sites = collections.defaultdict(list)   # f -> addresses naming it
    for a, ins in d.insns.items():
        for t, kind in ins.refs():
            if t in d.insns and t >= disasm.HEADER_END:
                f = owner(t)
                if f != owner(a) or t == f:
                    named[f].append((owner(a), bool(cov[a] & 1)))
                    sites[f].append(a)
    # a pointer known only from the two versions (vermap.moved_longs)
    # is an operand in bytes that aren't code yet, mostly dead code after
    # a table (D_00A83A, D_013838): it names nothing live
    BYTES = -1
    for a, t in d.ptrs.items():
        if t in d.insns:
            alone = a in d.moved and a not in d.longs | d.table_longs
            named[owner(t)].append((BYTES if alone else None, True))
            sites[owner(t)].append(a)
    # dead: unnamed, or named only by dead functions (to a fixpoint)
    dead = set(f for f in funcs if not ran[f] and not named[f])
    dead.add(BYTES)
    while True:
        more = set(f for f in funcs if not ran[f] and f not in dead and
                   named[f] and all(r in dead for r, _ in named[f]
                                    if r != f))
        if not more:
            break
        dead |= more
    dead.discard(BYTES)
    never = [f for f in funcs if not ran[f] and f not in dead]
    partly = [f for f in funcs if ran[f] and ran[f] < size[f]]

    # where each never-run chain starts: the instructions in functions
    # that ran (or the table longs) naming a function of the chain
    nev = set(never)
    entry = {}
    for f in never:
        seen, todo, out = set(), [f], set()
        while todo:
            g = todo.pop()
            if g in seen:
                continue
            seen.add(g)
            for (r, _), a in zip(named[g], sites[g]):
                if r is None or (r != BYTES and ran[r]):
                    out.add(a)
                elif r in nev:
                    todo.append(r)
        entry[f] = frozenset(out)
    chains = collections.defaultdict(list)
    for f in never:
        chains[entry[f]].append(f)

    def scripts(f):
        """The scripts that ran function f (where to look)."""
        w = [n for n, c in per.items() if c[f] & 1]
        return ', '.join(w[:3]) + (' and %d more' % (len(w) - 3)
                                   if len(w) > 3 else '')

    def site(a):
        if a in d.insns:
            f = owner(a)
            return '`%06X %s`%s in func_%06X (run by %s)' % (
                a, d.insns[a].format(), '' if cov[a] & 1 else ' (not run)',
                f, scripts(f) or 'nothing')
        return 'table long at %06X' % a

    def refs(f):
        out = []
        for r, live in named[f][:4]:
            out.append('a table' if r is None else 'bytes' if r == BYTES else
                       'func_%06X%s' % (r, '' if live else ' (not run)'))
        return ', '.join(out) + (', ...' if len(named[f]) > 4 else '')

    total = len(d.insns)
    un = sum(size[f] - ran[f] for f in funcs)
    lines = [
        '# Code no trace runs (%s)' % v,
        '',
        'Generated by `tools/unrun.py %s` from `build/%s/trace/` '
        '(`tools/trace_all.sh %s`); see its docstring for the groups.' %
        (v, v, v),
        '%d of %d instructions never run: %d in dead functions, %d in '
        'live functions never run, %d in functions that ran in part.' % (
            un, total, sum(size[f] for f in dead),
            sum(size[f] for f in never),
            sum(size[f] - ran[f] for f in partly)),
        '',
        '## Never run, live: by where they start (%d chains)' % len(chains),
        '',
        'Each chain is the never-run functions reached from the same',
        'places in code that ran (an instruction there that never ran,',
        'or that ran with another value) or in a table: what to trigger.',
        '',
    ]
    for k, fs in sorted(chains.items(),
                        key=lambda kv: -sum(size[f] for f in kv[1])):
        lines.append('- **%d instructions**: %s; from %s' % (
            sum(size[f] for f in fs),
            ', '.join('func_%06X' % f for f in sorted(fs)),
            '; '.join(site(a) for a in sorted(k)[:4]) or 'nowhere') +
            (' ...' if len(k) > 4 else ''))
    lines += ['', '## Dead (%d functions)' % len(dead), '',
              'Nothing that could run names them.', '',
              '| function | instructions | named by |', '|---|---|---|']
    for f in sorted(dead):
        lines.append('| func_%06X | %d | %s |' % (
            f, size[f], refs(f) or 'nothing'))
    lines += ['', '## Ran in part (%d functions)' % len(partly), '',
              '| function | not run | of |', '|---|---|---|']
    for f in sorted(partly, key=lambda f: (-(size[f] - ran[f]), f)):
        lines.append('| func_%06X | %d | %d |' % (
            f, size[f] - ran[f], size[f]))
    with open(OUT, 'w') as fo:
        fo.write('\n'.join(lines) + '\n')
    print('%d never run: %d dead, %d live never run (%d functions), %d in '
          'part' % (un, sum(size[f] for f in dead),
                    sum(size[f] for f in never), len(never),
                    sum(size[f] - ran[f] for f in partly)))


if __name__ == '__main__':
    main()
