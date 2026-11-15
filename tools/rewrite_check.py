#!/usr/bin/env python3
"""The rewritten routines against the original, call by call, over the
pad scripts (docs/PORT.md, "Rewriting").

    tools/rewrite_check.py [ADDR...] [--scripts A,B] [--frames N] [--calls N]

For each script, tools/mdtrace --snaps saves the registers and work RAM
where each routine (those named, else every one in crates/game/src/
rewritten/) is entered and where it returns, up to --calls calls a
script; then the port (target/release/megafuse --routines) runs the
routine from each entry state, and the results are compared as
tools/routine_check.py does: work RAM but the stack and what an
interrupt during the call changes, and the registers and flags the
callers read.  A routine that waits for the frame (it calls `idle`)
can't run on its own and is left out, as is one that takes arguments
on the stack.  Exits 1 if any call differs.
"""
import argparse
import glob
import os
import re
import shutil
import struct
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

TOP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(TOP, 'tools'))
from routine_check import FLAGS, NAMES, STACK, TIMING  # noqa: E402

REWRITTEN = os.path.join(TOP, 'crates', 'game', 'src', 'rewritten')


def rewritten():
    """{entry: waits} of the routines rewritten by hand."""
    out = {}
    for path in glob.glob(os.path.join(REWRITTEN, '**', '*.rs'),
                          recursive=True):
        text = open(path).read()
        for m in re.finditer(r'pub fn r_([0-9a-f]{6})\(', text):
            # the body: to the brace that closes the one after the head
            k = text.index('{', m.end())
            depth, j = 0, k
            while True:
                if text[j] == '{':
                    depth += 1
                elif text[j] == '}':
                    depth -= 1
                    if depth == 0:
                        break
                j += 1
            out[int(m.group(1), 16)] = 'self.idle()' in text[k:j]
    return out


def script_frames(path, default):
    n = default
    for line in open(path):
        m = re.match(r'#\s*frames\s+(\d+)', line)
        if m:
            n = min(n, int(m.group(1)))
    return n


def compare(exp, got, outs):
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
        elif o in NAMES:
            i = NAMES.index(o)
            if er[i] != gr[i]:
                diffs.append('%s %08X/%08X' % (o, er[i], gr[i]))
    ram = [i for i in range(STACK) if e[72 + i] != g[72 + i] and
           i not in TIMING]
    if ram:
        diffs.append('RAM %s%s' % (' '.join(
            '%04X:%02X/%02X' % (i, e[72 + i], g[72 + i]) for i in ram[:8]),
            ' ...' if len(ram) > 8 else ''))
    return diffs


def run_script(script, frames, calls, addrs, tmp):
    name = os.path.basename(script)[:-4]
    d = os.path.join(tmp, name)
    os.makedirs(d)
    lst = os.path.join(d, 'list')
    with open(lst, 'w') as f:
        f.write(''.join('%X\n' % a for a in addrs))
    subprocess.run(['nice', os.path.join(TOP, 'tools/mdtrace'), '--input',
                    script, '--frames', str(frames), '--snaps', lst,
                    '--snap-out', os.path.join(d, 's'), '--snap-limit',
                    str(calls), 'baserom.us.bin'], cwd=TOP, check=True,
                   stdout=subprocess.DEVNULL)
    jobs = []
    for a in addrs:
        k = 0
        while True:
            inp = os.path.join(d, 's%06X-%d-in.bin' % (a, k))
            exp = os.path.join(d, 's%06X-%d-out.bin' % (a, k))
            if not (os.path.exists(inp) and os.path.exists(exp)):
                break
            jobs.append((a, inp, exp, os.path.join(d, 'p%06X-%d.bin' % (a, k))))
            k += 1
    if not jobs:
        return name, []
    with open(os.path.join(d, 'jobs'), 'w') as f:
        for a, inp, _, got in jobs:
            f.write('%X %s %s\n' % (a, inp, got))
    port = os.environ.get('MEGAFUSE',
                          os.path.join(TOP, 'target/release/megafuse'))
    r = subprocess.run(['nice', port, '--routines', 'baserom.us.bin',
                        os.path.join(d, 'jobs')], cwd=TOP,
                       capture_output=True, text=True)
    said = {}
    for line in r.stdout.splitlines():
        f = line.split(' ', 3)
        if len(f) >= 3:
            said[f[1]] = (f[2], f[3] if len(f) > 3 else '')
    results = []
    for a, inp, exp, got in jobs:
        status, rest = said.get(got, ('error', 'no answer (%s)' % (
            r.stderr.strip().splitlines() or ['crashed'])[-1]))
        if status != 'ok':
            results.append((a, [rest]))
        else:
            results.append((a, compare(exp, got, rest.split())))
    return name, results


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('addrs', nargs='*')
    ap.add_argument('--scripts', help='comma-separated names in tools/inputs')
    ap.add_argument('--frames', type=int, default=6000)
    ap.add_argument('--calls', type=int, default=20)
    ap.add_argument('--jobs', type=int, default=os.cpu_count() or 4)
    ap.add_argument('--keep', action='store_true')
    a = ap.parse_args()
    known = rewritten()
    if a.addrs:
        addrs = [int(x, 16) for x in a.addrs]
    else:
        addrs = sorted(known)
    waits = [x for x in addrs if known.get(x)]
    addrs = [x for x in addrs if not known.get(x)]
    if a.scripts:
        scripts = [os.path.join(TOP, 'tools/inputs', s + '.txt')
                   for s in a.scripts.split(',')]
    else:
        scripts = sorted(glob.glob(os.path.join(TOP, 'tools/inputs/*.txt')))
    os.makedirs(os.path.join(TOP, 'build'), exist_ok=True)
    tmp = tempfile.mkdtemp(prefix='rewrite-', dir=os.path.join(TOP, 'build'))
    with ThreadPoolExecutor(a.jobs) as ex:
        done = list(ex.map(lambda s: run_script(
            s, script_frames(s, a.frames), a.calls, addrs, tmp), scripts))
    calls = {x: 0 for x in addrs}
    bad = {x: [] for x in addrs}
    alone = set()
    for name, results in done:
        for x, diffs in results:
            if diffs and 'not callable on its own' in diffs[0]:
                alone.add(x)
                continue
            calls[x] += 1
            if diffs:
                bad[x].append('%s: %s' % (name, '; '.join(diffs)))
    failed = 0
    for x in addrs:
        if bad[x]:
            failed += 1
            print('%06X: %d calls, %d differ' % (x, calls[x], len(bad[x])))
            for line in bad[x][:5]:
                print('    ' + line)
        elif x in alone:
            print('%06X: takes stack arguments (not run on its own)' % x)
        elif not calls[x]:
            print('%06X: never called' % x)
        else:
            print('%06X: %d calls, the same' % (x, calls[x]))
    for x in waits:
        print('%06X: waits for the frame (not run on its own)' % x)
    if a.keep:
        print('kept in %s' % tmp)
    else:
        shutil.rmtree(tmp)
    return 1 if failed else 0


if __name__ == '__main__':
    sys.exit(main())
