#!/usr/bin/env python3
"""Write a Mega Drive ROM's header checksum and ROM end.

The checksum (0x18E) is the 16-bit sum of the big-endian words from 0x200
to the end of the ROM, and the startup code (func_007AAE onwards) checks
it against the header: a wrong one hangs on a red screen.  The ROM end
(0x1A4) is the last byte's address.  For an unmodified build both are what
the ROM already has.

    tools/checksum.py ROM [--check]
"""
import struct
import sys


def checksum(rom):
    n = (len(rom) - 0x200) // 2
    return sum(struct.unpack_from('>%dH' % n, rom, 0x200)) & 0xFFFF


def main():
    path = sys.argv[1]
    rom = bytearray(open(path, 'rb').read())
    if len(rom) & 1:
        rom.append(0)
    want = checksum(rom)
    have = struct.unpack_from('>H', rom, 0x18E)[0]
    end = len(rom) - 1
    if '--check' in sys.argv:
        ok = want == have and struct.unpack_from('>I', rom, 0x1A4)[0] == end
        print('%s: checksum %04X, header %04X' % (path, want, have))
        return 0 if ok else 1
    struct.pack_into('>H', rom, 0x18E, want)
    struct.pack_into('>I', rom, 0x1A4, end)
    open(path, 'wb').write(rom)
    return 0


if __name__ == '__main__':
    sys.exit(main())
