#!/usr/bin/env python3
"""One routine of the port against the original, call by call.

    tools/routine_check.py ADDR [--script FILE] [--frames N] [--calls N]

Runs the original (tools/mdtrace --snap ADDR) for the script and frames,
which writes the registers and work RAM each time the routine at ADDR is
entered and returns; then the port's translation of it from each entry
state (megafuse --routine), and compares the results: work RAM (the
stack aside: the port's is its locals) and the registers and flags the
routine gives back (what tools/recomp found its callers read).  The
roadmap's check of the translation, phase 4.
"""
import argparse
import os
import struct
import subprocess
import sys
import tempfile

TOP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
NAMES = ['d%d' % k for k in range(8)] + ['a%d' % k for k in range(8)] + \
    ['pc', 'sr']
FLAGS = {'c': 1, 'v': 2, 'z': 4, 'n': 8, 'x': 16}
STACK = 0xFE00
# what an interrupt during the call changes in the original (the port runs
# the routine with them masked): the frame counters, the frame flag
TIMING = {0x805D, 0x80D4, 0x80D8, 0x9432, 0xA400, 0xA401, 0xAD06}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('addr')
    ap.add_argument('--script')
    ap.add_argument('--frames', type=int, default=3000)
    ap.add_argument('--calls', type=int, default=20)
    a = ap.parse_args()
    addr = int(a.addr, 16)
    tmp = tempfile.mkdtemp(prefix='routine-', dir=os.path.join(TOP, 'build'))
    cmd = ['nice', os.path.join(TOP, 'tools/mdtrace'), '--frames',
           str(a.frames), '--snap', '%X' % addr, '--snap-out',
           os.path.join(tmp, 's'), '--snap-limit', str(a.calls),
           'baserom.us.bin']
    if a.script:
        cmd[4:4] = ['--input', a.script]
    subprocess.run(cmd, cwd=TOP, check=True)
    bad = 0
    n = 0
    for k in range(a.calls):
        inp = os.path.join(tmp, 's%d-in.bin' % k)
        exp = os.path.join(tmp, 's%d-out.bin' % k)
        if not (os.path.exists(inp) and os.path.exists(exp)):
            break
        got = os.path.join(tmp, 'p%d.bin' % k)
        r = subprocess.run([os.path.join(TOP, 'target/release/megafuse'),
                            '--routine', 'baserom.us.bin', '%X' % addr, inp,
                            got], cwd=TOP, capture_output=True, text=True)
        if r.returncode:
            print('call %d: %s' % (k, (r.stderr.strip().splitlines() or
                                      ['failed'])[-1]))
            bad += 1
            continue
        outs = r.stdout.split()
        e = open(exp, 'rb').read()
        g = open(got, 'rb').read()
        er = struct.unpack('>18I', e[:72])
        gr = struct.unpack('>18I', g[:72])
        diffs = []
        for o in outs:
            if o in FLAGS:
                if bool(er[17] & FLAGS[o]) != bool(gr[17] & FLAGS[o]):
                    diffs.append('%s %d/%d' % (o, bool(er[17] & FLAGS[o]),
                                               bool(gr[17] & FLAGS[o])))
            else:
                i = NAMES.index(o)
                if er[i] != gr[i]:
                    diffs.append('%s %08X/%08X' % (o, er[i], gr[i]))
        ram = [i for i in range(STACK) if e[72 + i] != g[72 + i] and
               i not in TIMING]
        if ram:
            diffs.append('RAM %s%s' % (' '.join(
                '%04X:%02X/%02X' % (i, e[72 + i], g[72 + i]) for i in ram[:10]),
                ' ...' if len(ram) > 10 else ''))
        n += 1
        if diffs:
            bad += 1
            print('call %d: %s' % (k, '; '.join(diffs)))
    print('%06X: %d calls, %d differ (kept in %s)' % (addr, n, bad, tmp))
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
