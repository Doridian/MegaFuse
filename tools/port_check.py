#!/usr/bin/env python3
"""The port against the original: same pad script, same RAM and screens?

    tools/port_check.py [SCRIPT...] [--frames N] [--every N] [--window N]

Runs the port (target/release/megafuse, `cargo build --release`) and the
original in Genesis Plus GX (tools/mdtrace) on each pad script
(tools/inputs/, all by default) and compares work RAM every --every
frames.  The port keeps the console's time (docs/PORT.md, "Time"), so
its frames are the original's; --window N compares each with the
original's closest within N frames instead (0 by default).  Not
compared: the stack (0xFE00 on: the port's stack is its locals).
Reports, per script, the first frame that differs and how, and the
screens there (pixels that differ); tools/timeline_check.py finds the
instruction.
"""
import argparse
import os
import struct
import subprocess
import sys
import tempfile
import zlib

TOP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STACK = 0xFE00

def read_png(path):
    d = open(path, 'rb').read()
    i = 8
    idat = b''
    w = h = 0
    while i < len(d):
        n = struct.unpack('>I', d[i:i + 4])[0]
        t, c = d[i + 4:i + 8], d[i + 8:i + 8 + n]
        if t == b'IHDR':
            w, h = struct.unpack('>II', c[:8])
        elif t == b'IDAT':
            idat += c
        i += 12 + n
    raw = zlib.decompress(idat)
    stride = w * 3 + 1
    rows = []
    prev = bytearray(w * 3)
    for y in range(h):
        f = raw[y * stride]
        line = bytearray(raw[y * stride + 1:(y + 1) * stride])
        for x in range(len(line)):
            a = line[x - 3] if x >= 3 else 0
            b = prev[x]
            c = prev[x - 3] if x >= 3 else 0
            if f == 1:
                line[x] = (line[x] + a) & 0xFF
            elif f == 2:
                line[x] = (line[x] + b) & 0xFF
            elif f == 3:
                line[x] = (line[x] + (a + b) // 2) & 0xFF
            elif f == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pr = a if pa <= pb and pa <= pc else b if pb <= pc else c
                line[x] = (line[x] + pr) & 0xFF
        rows.append(bytes(line))
        prev = line
    return w, h, rows


def ram_diff(a, b):
    return [i for i in range(STACK) if a[i] != b[i]]


def check(script, frames, every, window, tmp):
    name = os.path.basename(script)[:-4] if script else 'none'
    port = os.path.join(tmp, name + '-p')
    orig = os.path.join(tmp, name + '-o')
    cmd = [os.path.join(TOP, 'target/release/megafuse'), 'baserom.us.bin',
           '--frames', str(frames), '--every', '1', '--ram', port + 'r',
           '--shot', port + 's']
    if script:
        cmd[2:2] = ['--input', script]
    r = subprocess.run(cmd, cwd=TOP, capture_output=True, text=True)
    crashed = r.returncode != 0 and r.stderr.strip().splitlines()[-2:]
    mt = ['nice', os.path.join(TOP, 'tools/mdtrace'), '--frames',
          str(frames + window), '--ram', orig + 'r', '--shot', orig + 's',
          '--shot-every', '1', 'baserom.us.bin']
    if script:
        mt[4:4] = ['--input', script]
        if '# teamplayer\n' in open(script).read():
            mt.insert(4, '--teamplayer')
    subprocess.run(mt, cwd=TOP, check=True, capture_output=True)
    last = 0
    for f in range(every, frames + 1, every):
        p = '%sr%06d.bin' % (port, f)
        if not os.path.exists(p):
            return name, last, 'the port stopped: %s' % (
                ' '.join(crashed) if crashed else 'no dump')
        b = open(p, 'rb').read()
        best = None
        for s in range(-window, window + 1):
            q = '%sr%06d.bin' % (orig, f + s)
            if f + s < 1 or not os.path.exists(q):
                continue
            d = ram_diff(open(q, 'rb').read(), b)
            if best is None or len(d) < len(best[1]):
                best = (s, d, q)
        s, d, q = best
        if d:
            a = open(q, 'rb').read()
            what = ' '.join('%04X:%02X/%02X' % (i, a[i], b[i]) for i in d[:12])
            pix = screens(orig, port, f, s)
            why = 'RAM: %d bytes (best at %+d frames) %s%s; %s' % (
                len(d), s, what, ' ...' if len(d) > 12 else '', pix)
            return name, last, why
        last = f
    return name, last, None


def screens(orig, port, f, s):
    try:
        w1, h1, a = read_png('%ss%06d.png' % (orig, f + s))
        w2, h2, b = read_png('%ss%06d.png' % (port, f))
    except (OSError, zlib.error):
        return 'no screens'
    if (w1, h1) != (w2, h2):
        return 'screens %dx%d and %dx%d' % (w1, h1, w2, h2)
    n = sum(1 for y in range(h1) for x in range(w1)
            if a[y][3 * x:3 * x + 3] != b[y][3 * x:3 * x + 3])
    return 'screen: %d pixels differ' % n


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('scripts', nargs='*')
    ap.add_argument('--frames', type=int, default=3000)
    ap.add_argument('--every', type=int, default=20)
    ap.add_argument('--window', type=int, default=0)
    a = ap.parse_args()
    scripts = a.scripts or sorted(
        os.path.join(TOP, 'tools/inputs', x)
        for x in os.listdir(os.path.join(TOP, 'tools/inputs'))
        if x.endswith('.txt'))
    if a.scripts == ['none']:
        scripts = [None]
    tmp = tempfile.mkdtemp(prefix='port-', dir=os.path.join(TOP, 'build'))
    bad = 0
    for sc in scripts:
        name, last, why = check(sc, a.frames, a.every, a.window, tmp)
        if why:
            bad += 1
            print('%-16s same to %d, then %s' % (name, last, why))
        else:
            print('%-16s ok to %d' % (name, last))
    print('kept in %s' % tmp)
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
