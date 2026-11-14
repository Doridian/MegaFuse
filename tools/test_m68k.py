#!/usr/bin/env python3
"""ROM-free self-test of tools/m68k.py against gas.

Decodes every 16-bit first word, with several sets of extension words
(ordinary, negative, all zero), formats each instruction the decoder accepts
the way tools/disasm.py does (branch and pc-relative targets as `.`-relative
expressions), assembles the lot with gas and checks that every instruction
comes back as the same bytes.  Anything gas would spell differently has to
be refused (or marked raw) by the decoder instead.

    tools/test_m68k.py [--as PATH]
"""
import argparse
import os
import struct
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import m68k  # noqa: E402

EXTS = [
    '0012003400560078009a00bc',
    'fff0800280040006000800aa',
    '00fe12347ffe0010ffff0002',
    '000000000000000000000000',
    '00000000ff00000000000000',
]
ASFLAGS = ['-m68000', '--register-prefix-optional']


def rel(a, at):
    return ((a - at + 0x800000) & 0xffffff) - 0x800000


def main():
    ap = argparse.ArgumentParser()
    here = os.path.dirname(os.path.abspath(__file__))
    ap.add_argument('--as', dest='gas',
                    default=os.path.join(here, 'binutils/bin/m68k-elf-as'))
    ap.add_argument('--objcopy', default=None)
    a = ap.parse_args()
    objcopy = a.objcopy or a.gas[:-2] + 'objcopy'
    lines = ['\t.text', '\t.opt noquick']
    ref = bytearray()
    starts = []
    raw = 0
    for e in EXTS:
        e = bytes.fromhex(e)
        for w in range(0x10000):
            at = len(ref)
            rom = bytes(at) + struct.pack('>H', w) + e
            ins = m68k.decode(rom, at)
            if ins is None:
                continue
            if ins.raw:
                raw += 1
                text = '.short ' + ','.join(
                    '0x%04X' % v for v in
                    struct.unpack('>%dH' % (ins.size // 2),
                                  rom[at:at + ins.size]))
            else:
                text = ins.format(
                    lambda t, k: '(.+%d)' % rel(t, at)
                    if k in ('pc', 'branch') else None)
            lines.append('\t' + text)
            starts.append(at)
            ref += rom[at:at + ins.size]
    with tempfile.TemporaryDirectory() as d:
        s, o, b = (os.path.join(d, n) for n in ('t.s', 't.o', 't.bin'))
        open(s, 'w').write('\n'.join(lines) + '\n')
        subprocess.run([a.gas] + ASFLAGS + ['-o', o, s], check=True)
        subprocess.run([objcopy, '-O', 'binary', '-j', '.text', o, b],
                       check=True)
        out = open(b, 'rb').read()
    bad = 0
    starts.append(len(ref))
    for i in range(len(starts) - 1):
        x, y = starts[i], starts[i + 1]
        if out[x:y] != ref[x:y]:
            if bad < 20:
                print('mismatch: %-40s ours %s gas %s' % (
                    lines[i + 2].strip(), ref[x:y].hex(), out[x:y].hex()))
            bad += 1
    print('%d instructions (%d raw), %d mismatches' % (
        len(starts) - 1, raw, bad))
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
