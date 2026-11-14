#!/usr/bin/env python3
"""Where each us address is in another version.

    tools/vermap.py VERSION        print the map (us lo, us hi, VERSION lo)

us is the reference version: its names (`func_X`, `D_X`, X the address in
us) are the names of the same things in eu, so one source can build both
(docs/LAYOUT.md, "US and EU").  The map is a list of ranges that sit at
a fixed distance in the two ROMs, found from the two disassemblies alone:

1. Both versions are analysed (tools/disasm.py) and their instructions,
   in address order, aligned on what they are with the numbers masked
   (difflib): a pointer that moved reads the same.
2. Aligned instructions at the same distance make a range, from the
   first one's start to the last one's end, the data between them
   included (the same length in both, since the distance is the same).
   Where the distance changes, what lies between is in one version only.
   The header and the data after the code (from the end of the zeros
   that follow it) map as a whole where both versions have the same
   bytes there.

In eu: everything up to 0x7C4A is where it is in us; us then has a
`movem.l (0x0).w,a7` eu doesn't (6 bytes), and from 0x7C50 to the end of
the code eu is 6 bytes lower.  us also has 14 bytes after its last
function (a copy of its tail) where eu has zeros.  The data is the same.
"""
import bisect
import difflib
import os
import re
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import disasm  # noqa: E402

REF = 'us'
NUM = re.compile(r'(0x[0-9A-Fa-f]+|\b[0-9A-F]{6}\b)')


def token(ins):
    return NUM.sub('N', ins.format())


def data_start(d):
    """Where the data starts: the end of the zeros after the code."""
    for lo, hi in d.gaps:
        if lo >= d.code_limit - 0x4000 and hi < d.end:
            return hi
    return None


def build(other, ref=None, oth=None):
    """[(ref lo, ref hi, other lo)], sorted, for ref=us and other (their
    analyses, if done already)."""
    ref = ref or disasm.analyze(REF, versions=False)
    oth = oth or disasm.analyze(other, versions=False)
    ra = sorted(ref.insns)
    oa = sorted(oth.insns)
    sm = difflib.SequenceMatcher(
        None, [token(ref.insns[a]) for a in ra],
        [token(oth.insns[a]) for a in oa], autojunk=False)
    ranges = []                  # [ref lo, ref hi, delta]
    for i, j, n in sm.get_matching_blocks():
        for k in range(n):
            a, b = ra[i + k], oa[j + k]
            hi = a + ref.insns[a].size
            if ranges and ranges[-1][2] == b - a:
                ranges[-1][1] = hi
            else:
                ranges.append([a, hi, b - a])
    # the header, up to the first range, where it is the same distance
    if ranges and ranges[0][2] == 0:
        ranges[0][0] = 0
    # the data after the code
    rs, os_ = data_start(ref), data_start(oth)
    if rs is not None and rs == os_ and ref.rom[rs:] == oth.rom[os_:]:
        ranges.append([rs, ref.end, os_ - rs])
    return [(lo, hi, lo + dl) for lo, hi, dl in ranges]


def numbers(ref, oth, vm):
    """Long immediates that are numbers: the same instruction holds the
    same value in both versions, while the address it would be moved
    (`move.l #0xFE00,(0x8A5E).w` sets 0x8A60 to 0xFE; D_00FE00 is 6
    bytes lower in eu).  They go to each one's Disasm.numbers."""
    for a, ins in ref.insns.items():
        b = vm.to_other(a)
        o = oth.insns.get(b)
        if o is None or o.mn != ins.mn:
            continue
        for x, y in zip(ins.ops, o.ops):
            if x.kind == 'imm' and y.kind == 'imm' and x.size == 'l' and \
                    x.val == y.val and ref.in_rom(x.val):
                t = vm.to_other(x.val)
                if t is not None and t != x.val:
                    ref.numbers.add(a)
                    oth.numbers.add(b)


def moved_longs(ref, oth, vm):
    """Longs in the data of the code area that are pointers by the
    versions' say: the value in the other version is where the map puts
    the value in ref, and they differ (the address moved).  A number
    doesn't move with the code.  Any alignment (0x1B7BA's records hold
    them at odd offsets).  They go to each one's Disasm.moved."""
    lim = min(ref.code_limit, ref.end - 3)
    for a in range(disasm.HEADER_END, lim):
        if any(a + k in ref.owner for k in range(4)):
            continue
        b = vm.to_other(a)
        if b is None or vm.to_other(a + 3) != b + 3:
            continue
        v = struct.unpack_from('>I', ref.rom, a)[0]
        w = struct.unpack_from('>I', oth.rom, b)[0]
        if v != w and vm.to_other(v) == w:
            ref.moved.add(a)
            oth.moved.add(b)


class VerMap:
    """ref <-> other addresses through build()'s ranges."""

    def __init__(self, ranges):
        self.ranges = ranges
        self.ref_los = [r[0] for r in ranges]
        self.inv = sorted((o, o + hi - lo, lo) for lo, hi, o in ranges)
        self.oth_los = [r[0] for r in self.inv]

    @staticmethod
    def _look(rs, los, a):
        i = bisect.bisect_right(los, a) - 1
        if i >= 0 and rs[i][0] <= a < rs[i][1]:
            return rs[i][2] + a - rs[i][0]
        return None

    def to_other(self, a):
        return self._look(self.ranges, self.ref_los, a)

    def to_ref(self, a):
        return self._look(self.inv, self.oth_los, a)


def main():
    if len(sys.argv) != 2 or sys.argv[1] == REF:
        sys.exit(__doc__.split('\n\n')[1])
    for lo, hi, o in build(sys.argv[1]):
        print('%06X %06X %06X  %+d' % (lo, hi, o, o - lo))


if __name__ == '__main__':
    main()
