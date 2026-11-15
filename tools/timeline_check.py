#!/usr/bin/env python3
"""The port's time against the original's, instruction by instruction.

    tools/timeline_check.py [SCRIPT] [--frames N] [--context N] [--z]

Runs the port (target/release/megafuse) and the original in Genesis Plus
GX (tools/mdtrace) with --timeline: for each frame, how many
instructions the 68000 ran and a hash of their (PC, clock), the clock in
master clocks from power-on.  The first frame that differs is run again
in full on both, and the first instruction where they part is shown
with the ones before it.  When the port's time is the console's (as
Genesis Plus GX keeps it), every frame is the same: the same
interrupts at the same instructions, the same RAM (docs/PORT.md,
"Time").  --z: the Z80's instead (its PC, the same clock).
"""
import argparse
import os
import subprocess
import sys
import tempfile

TOP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def run(script, frames, out, at=None, z=False):
    flag = '--ztimeline' if z else '--timeline'
    port = [os.path.join(TOP, 'target/release/megafuse'), 'baserom.us.bin',
            '--frames', str(frames), '--every', '0', flag,
            out + '-port.txt']
    orig = ['nice', os.path.join(TOP, 'tools/mdtrace'), '--frames',
            str(frames), flag, out + '-orig.txt', 'baserom.us.bin']
    if at is not None:
        port += ['--timeline-at', str(at)]
        orig[4:4] = ['--timeline-at', str(at)]
    if script:
        port[2:2] = ['--input', script]
        orig[4:4] = ['--input', script]
        text = open(script).read()
        if '# teamplayer\n' in text:
            orig.insert(4, '--teamplayer')
        if '# wayplay\n' in text:
            orig.insert(4, '--wayplay')
    p = subprocess.run(['nice'] + port, cwd=TOP, capture_output=True,
                       text=True)
    subprocess.run(orig, cwd=TOP, check=True, capture_output=True)
    if p.returncode != 0:
        print('the port stopped:', '\n'.join(p.stderr.strip().splitlines()
                                             [-3:]))
    return out + '-port.txt', out + '-orig.txt'


def frames_of(path):
    out = {}
    for line in open(path):
        if line.startswith('FRAME'):
            _, f, n, h = line.split()
            out[int(f)] = (int(n), h)
    return out


def insns_of(path):
    return [(int(f[0], 16), int(f[1])) for f in
            (line.split() for line in open(path)
             if not line.startswith('FRAME'))]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('script', nargs='?')
    ap.add_argument('--frames', type=int, default=3000)
    ap.add_argument('--context', type=int, default=12)
    ap.add_argument('--z', action='store_true')
    a = ap.parse_args()
    tmp = tempfile.mkdtemp(prefix='timeline-', dir=os.path.join(TOP,
                                                                'build'))
    out = os.path.join(tmp, 'tl')
    pp, op = run(a.script, a.frames, out, z=a.z)
    p, o = frames_of(pp), frames_of(op)
    bad = next((f for f in sorted(o) if p.get(f) != o[f]), None)
    if bad is None:
        print('the same to frame %d' % (max(o) + 1 if o else 0))
        return 0
    same = bad
    print('the same to frame %d; frame %d: port %s, original %s'
          % (same, bad, p.get(bad), o[bad]))
    pp, op = run(a.script, bad + 1, out + '-at', bad, a.z)
    pi, oi = insns_of(pp), insns_of(op)
    k = next((k for k in range(min(len(pi), len(oi))) if pi[k] != oi[k]),
             min(len(pi), len(oi)))
    print('instruction %d of the frame (port, original):' % k)
    for j in range(max(0, k - a.context), min(k + 4, max(len(pi),
                                                          len(oi)))):
        x = '%06X %d' % pi[j] if j < len(pi) else '-'
        y = '%06X %d' % oi[j] if j < len(oi) else '-'
        d = ''
        if j < len(pi) and j < len(oi) and pi[j][0] == oi[j][0]:
            d = '%+d' % (pi[j][1] - oi[j][1])
        print('  %s %-22s %-22s %s' % ('>' if j == k else ' ', x, y, d))
    print('kept in %s' % tmp)
    return 1


if __name__ == '__main__':
    sys.exit(main())
