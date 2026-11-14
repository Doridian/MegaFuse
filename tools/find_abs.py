#!/usr/bin/env python3
"""List what in the disassembly may be a ROM address left as a number.

    tools/find_abs.py VERSION [--list N]

The static side of the SHIFT check (tools/shift_check.py runs only what the
traces run).  It runs tools/disasm.py's analysis (no output written) and
reports, by kind:

  imm       a long immediate that names a function or data label in the
            code area but isn't taken as an address (`move.l #x,d0`,
            `cmpi.l`, ...)
  disp      a displacement off an address register in the code area's range
            that isn't taken as an address (no trace read data there)
  raw       an instruction written as words (`.short`) with an immediate
            that names a label
  data-long a long in the data between code, not taken as a pointer, whose
            value is a label in the code area
  data-word the same for a word: a function's address below 0x10000

Bytes a script walk decoded (SCRIPTS in tools/disasm.py) aren't listed:
their pointers are known.  Not everything listed is an address (a number can equal a label by
chance); each needs a look.  Addresses into the data (0x40000 on) aren't
listed: the data doesn't move in a SHIFT build yet.
"""
import argparse
import collections
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import disasm  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('version')
    ap.add_argument('--list', type=int, default=10)
    a = ap.parse_args()
    rom = open(os.path.join(disasm.TOP, 'baserom.%s.bin' % a.version),
               'rb').read()
    extra, data, longs = disasm.load_known(a.version)
    d = disasm.Disasm(rom, extra, data, longs)
    d.biased = dict(disasm.BIASED.get(a.version, {}))
    d.scripts = disasm.SCRIPTS.get(a.version, [])
    d.trace_all()
    d.speculate_all()
    d.collect_refs()
    labels = d.labels() if hasattr(d, 'labels') else {}
    named = set(k for k in labels if k >= disasm.HEADER_END)
    lim = d.code_limit
    found = collections.defaultdict(list)
    for at, ins in sorted(d.insns.items()):
        for o in ins.ops:
            if o.kind == 'imm' and o.size == 'l' and o.val in named and \
                    o.val < lim and not d.imm_pointer(ins, o.val):
                found['raw' if ins.raw else 'imm'].append(
                    '%06X  %s' % (at, ins.format()))
            if o.kind == 'disp' and disasm.HEADER_END <= o.val < lim and \
                    o.val in named and not d.disp_pointer(o.val):
                found['disp'].append('%06X  %s' % (at, ins.format()))
    funcs = set(f for f in d.funcs if f in d.insns)
    covered = set(d.script_bytes)   # decoded: its longs are known
    for p in d.ptrs:
        covered.update(range(p, p + 4))
    for k in range(disasm.HEADER_END, lim - 3, 2):
        if any(k + j in d.owner or k + j in covered for j in range(4)):
            continue
        v = struct.unpack_from('>I', rom, k)[0]
        if v in named and v < lim:
            found['data-long'].append('%06X  .long 0x%X' % (k, v))
    for k in range(disasm.HEADER_END, lim - 1, 2):
        if any(k + j in d.owner or k + j in covered for j in range(2)):
            continue
        v = struct.unpack_from('>H', rom, k)[0]
        if v in funcs:
            found['data-word'].append('%06X  .short 0x%X' % (k, v))
    for kind in ('imm', 'disp', 'raw', 'data-long', 'data-word'):
        print('%-10s %d' % (kind, len(found[kind])))
        for line in found[kind][:a.list]:
            print('    ' + line)


if __name__ == '__main__':
    main()
