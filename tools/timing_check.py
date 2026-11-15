#!/usr/bin/env python3
"""tools/recomp/timing.py against what each instruction took in Genesis
Plus GX (tools/mdtrace --cycles).

    tools/timing_check.py [CYCLES...] [--examples N]

CYCLES are mdtrace --cycles files (by default it makes one: the
attract mode, 3000 frames).  For each instruction traced, the costs
seen (68000 cycles; the bus refresh's 2 taken off) against timing.py's;
the mismatches grouped by form (mnemonic, size and operand kinds), most
instructions first.  An instruction seen at more than one cost is
`dynamic` (a branch taken or not, a shift's count, MULU/DIVU's operands,
a wait on the VDP or the Z80's bus): those are listed apart, with what
timing.py says.
"""
import argparse
import collections
import os
import subprocess
import sys
import tempfile

TOP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(TOP, 'tools'))
import disasm  # noqa: E402
from recomp import timing  # noqa: E402

def load(paths):
    """pc -> Counter((cycles, skipped a refresh) -> count)."""
    seen = collections.defaultdict(collections.Counter)
    for p in paths:
        for line in open(p):
            pc, cost, skip, n = line.split()
            seen[int(pc, 16)][(int(cost) / 7, skip == '1')] += int(n)
    return seen


def allowed(c):
    """The times an instruction of Cost c may take (None: any of its
    family's, which the emitter works out as it runs)."""
    t = c.base + c.pre
    return {
        None: {t},
        'bcc.s': {t, t + 2},
        'bcc.w': {t, t + 2},
        'dbcc': {t, t + 2, t + 4},
        'scc': {t, t + 2},
        'bit': {t, t + 2},
        'shift': {t + 2 * n for n in range(64)},
        'mulu': {t + 38 + 2 * n for n in range(17)},
        'muls': {t + 38 + 2 * n for n in range(17)},
        'divu': None, 'divs': None,
    }[c.dyn]


def form(i):
    return '%s %s' % (i.mn, ','.join(o.kind for o in i.ops))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('cycles', nargs='*')
    ap.add_argument('--examples', type=int, default=3)
    a = ap.parse_args()
    paths = a.cycles
    if not paths:
        tmp = tempfile.mkdtemp(prefix='timing-', dir=os.path.join(TOP,
                                                                  'build'))
        paths = [os.path.join(tmp, 'none.txt')]
        subprocess.run(['nice', os.path.join(TOP, 'tools/mdtrace'),
                        '--frames', '3000', '--cycles', paths[0],
                        'baserom.us.bin'], cwd=TOP, check=True,
                       capture_output=True)
    seen = load(paths)
    d = disasm.analyze('us')
    wrong = collections.defaultdict(list)
    waits = collections.defaultdict(list)
    right = total = 0
    for pc, costs in sorted(seen.items()):
        i = d.insns.get(pc)
        if i is None:
            continue
        total += 1
        c = timing.cost(i)
        ok = allowed(c)
        times = set(t for t, _ in costs)
        skips = set(sk for _, sk in costs)
        bad_skip = True in skips and not c.skip
        if ok is not None and not times <= ok and c.dyn is None and \
                min(times) in ok:
            # more than the table at times: the bus made it wait
            waits[form(i)].append((pc, costs))
            continue
        if (ok is not None and not times <= ok) or bad_skip:
            got = ' '.join('%g%s' % (t, '*' if sk else '')
                           for t, sk in sorted(costs))
            want = ' '.join('%g' % t for t in sorted(ok or [])[:4])
            wrong[(form(i), got, want + ('*' if c.skip else ''))].append(
                (pc, i))
            continue
        right += 1
    print('%d instructions traced: %d right, %d wrong, %d waited'
          % (total, right, sum(len(v) for v in wrong.values()),
             sum(len(v) for v in waits.values())))
    print('\nwrong (form: GPGX vs timing.py, * a refresh skipped):')
    for (f, got, want), v in sorted(wrong.items(), key=lambda x: -len(x[1])):
        ex = '  '.join('%06X' % pc for pc, i in v[:a.examples])
        print('  %-28s %-16s %-16s %4d  %s' % (f, got, want, len(v), ex))
    print('\nwaited (form: instructions, e.g.):')
    for f, v in sorted(waits.items(), key=lambda x: -len(x[1])):
        pc, costs = v[0]
        c = ' '.join('%g:%d' % (t, n) for (t, _), n in sorted(costs.items()))
        print('  %-28s %4d  %06X %s' % (f, len(v), pc, c))


if __name__ == '__main__':
    main()
