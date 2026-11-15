#!/usr/bin/env python3
"""The port against the original: same pad script, same RAM and screens?

    tools/port_check.py [SCRIPT...] [--frames N]

Runs the port (target/release/megafuse, `cargo build --release`) and the
original in Genesis Plus GX (tools/mdtrace) on each pad script
(tools/inputs/, all by default; `none` for no input) with --hashes: each
frame's CRC-32 of work RAM (to the stack, 0xFE00: the port's stack is
its locals) and of the picture.  The port keeps the console's time
(docs/PORT.md, "Time"), so every frame should be the same.  The first
that isn't is run again on both and dumped: the RAM bytes that differ,
the pixels (and lines) that do.  tools/timeline_check.py finds the
instruction where a run parts.
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
        for x in range(len(line) if f else 0):
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


def run(script, frames, out, extra_port, extra_orig):
    port = [os.path.join(TOP, 'target/release/megafuse'), 'baserom.us.bin',
            '--frames', str(frames), '--every', '0'] + extra_port
    orig = ['nice', os.path.join(TOP, 'tools/mdtrace'), '--frames',
            str(frames)] + extra_orig + ['baserom.us.bin']
    if script:
        port[2:2] = ['--input', script]
        orig[4:4] = ['--input', script]
        text = open(script).read()
        if '# teamplayer\n' in text:
            orig.insert(4, '--teamplayer')
        if '# wayplay\n' in text:
            orig.insert(4, '--wayplay')
    r = subprocess.run(port, cwd=TOP, capture_output=True, text=True)
    subprocess.run(orig, cwd=TOP, check=True, capture_output=True)
    return r.returncode == 0 or r.stderr.strip().splitlines()[-2:]


def hashes(path):
    out = {}
    for line in open(path):
        f, ram, screen = line.split()
        out[int(f)] = (ram, screen)
    return out


def check(script, frames, tmp):
    """(name, frames the same, why the next isn't)."""
    name = os.path.basename(script)[:-4] if script else 'none'
    base = os.path.join(tmp, name)
    ok = run(script, frames, base, ['--hashes', base + '-p.txt'],
             ['--hashes', base + '-o.txt'])
    p, o = hashes(base + '-p.txt'), hashes(base + '-o.txt')
    bad = next((f for f in range(frames) if p.get(f) != o.get(f)), None)
    if bad is None:
        return name, frames, None
    if bad not in p:
        return name, bad, 'the port stopped: %s' % (
            ' '.join(ok) if ok is not True else 'no frame')
    # that frame (frame `bad` is the one run as the bad+1st), dumped
    n = bad + 1
    run(script, n, base, ['--every', str(n), '--ram', base + '-pr',
                          '--shot', base + '-ps'],
        ['--ram', base + '-or', '--shot', base + '-os', '--shot-every',
         str(n)])
    a = open('%s-or%06d.bin' % (base, n), 'rb').read()
    b = open('%s-pr%06d.bin' % (base, n), 'rb').read()
    d = ram_diff(a, b)
    why = []
    if d:
        what = ' '.join('%04X:%02X/%02X' % (i, a[i], b[i]) for i in d[:12])
        why.append('RAM: %d bytes %s%s' % (len(d), what,
                                           ' ...' if len(d) > 12 else ''))
    pix = screens(base + '-o', base + '-p', n, 0)
    if pix:
        why.append(pix)
    return name, bad, '; '.join(why) or 'hashes differ'


def screens(orig, port, f, s):
    try:
        w1, h1, a = read_png('%ss%06d.png' % (orig, f + s))
        w2, h2, b = read_png('%ss%06d.png' % (port, f))
    except (OSError, zlib.error):
        return 'no screens'
    if (w1, h1) != (w2, h2):
        return 'screens %dx%d and %dx%d' % (w1, h1, w2, h2)
    if a == b:
        return None
    rows = [y for y in range(h1) if a[y] != b[y]]
    n = sum(1 for y in rows for x in range(w1)
            if a[y][3 * x:3 * x + 3] != b[y][3 * x:3 * x + 3])
    return 'screen: %d pixels differ, lines %d-%d' % (n, rows[0], rows[-1])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('scripts', nargs='*')
    ap.add_argument('--frames', type=int, default=3000)
    a = ap.parse_args()
    scripts = a.scripts or sorted(
        os.path.join(TOP, 'tools/inputs', x)
        for x in os.listdir(os.path.join(TOP, 'tools/inputs'))
        if x.endswith('.txt'))
    scripts = [None if x == 'none' else x for x in scripts]
    tmp = tempfile.mkdtemp(prefix='port-', dir=os.path.join(TOP, 'build'))
    bad = 0
    for sc in scripts:
        name, last, why = check(sc, a.frames, tmp)
        if why:
            bad += 1
            print('%-16s same to frame %d, then %s' % (name, last, why))
        else:
            print('%-16s ok to frame %d' % (name, last))
    print('kept in %s' % tmp)
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
