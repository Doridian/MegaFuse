#!/usr/bin/env python3
"""Tile PNG screenshots (tools/mdtrace --shot) into one contact sheet.

    tools/sheet.py OUT.png [--cols N] SHOT.png...
"""
import argparse
import struct
import zlib


def read_png(path):
    """8-bit RGB PNGs as mdtrace writes them (filter 0 on every row)."""
    d = open(path, 'rb').read()
    i = 8
    idat = b''
    while i < len(d):
        n = struct.unpack('>I', d[i:i + 4])[0]
        t, c = d[i + 4:i + 8], d[i + 8:i + 8 + n]
        if t == b'IHDR':
            w, h = struct.unpack('>II', c[:8])
        elif t == b'IDAT':
            idat += c
        i += 12 + n
    raw = zlib.decompress(idat)
    s = w * 3 + 1
    return w, h, [raw[y * s + 1:(y + 1) * s] for y in range(h)]


def chunk(t, c):
    return struct.pack('>I', len(c)) + t + c + \
        struct.pack('>I', zlib.crc32(t + c))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('out')
    ap.add_argument('shots', nargs='+')
    ap.add_argument('--cols', type=int, default=5)
    a = ap.parse_args()
    imgs = [read_png(p) for p in a.shots]
    w = max(i[0] for i in imgs)
    h = max(i[1] for i in imgs)
    rows = (len(imgs) + a.cols - 1) // a.cols
    W, H = w * a.cols, h * rows
    out = [bytearray(W * 3) for _ in range(H)]
    for k, (iw, ih, rws) in enumerate(imgs):
        ox, oy = (k % a.cols) * w, (k // a.cols) * h
        for y in range(ih):
            out[oy + y][ox * 3:ox * 3 + iw * 3] = rws[y]
    raw = b''.join(b'\x00' + bytes(r) for r in out)
    open(a.out, 'wb').write(
        b'\x89PNG\r\n\x1a\n' +
        chunk(b'IHDR', struct.pack('>IIBBBBB', W, H, 8, 2, 0, 0, 0)) +
        chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b''))


if __name__ == '__main__':
    main()
