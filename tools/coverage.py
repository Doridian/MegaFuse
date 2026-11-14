#!/usr/bin/env python3
"""Compare runtime coverage (tools/mdtrace --cover) with tools/disasm.py.

    tools/coverage.py VERSION COVER [--list N] [--write]

Reports:
  - executed: ROM bytes that ran as instruction starts, and how much of the
    code the disassembler found the trace has confirmed;
  - missed: instruction starts that ran but the disassembler left as data
    (or found inside another instruction);
  - read-in-code: bytes read as data inside what the disassembler calls code
    (a table it decoded as instructions, or code read as data on purpose);
  - unconfirmed: code found by speculation that never ran (not wrong, just
    not evidence).
--write regenerates code.VERSION.txt, data.VERSION.txt and
longs.VERSION.txt from the trace (what tools/disasm.py takes as evidence): see write_known.
"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import disasm  # noqa: E402

TOP = disasm.TOP


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('version')
    ap.add_argument('cover', nargs='+')
    ap.add_argument('--list', type=int, default=20)
    ap.add_argument('--write', action='store_true')
    a = ap.parse_args()
    rom = open(os.path.join(TOP, 'baserom.%s.bin' % a.version), 'rb').read()
    cov = bytearray(len(rom))
    for p in a.cover:
        c = open(p, 'rb').read()
        for i in range(min(len(c), len(rom))):
            cov[i] |= c[i]
    if a.write:
        extra, data = [], []      # recompute both from the trace alone
    else:
        extra, data, _ = disasm.load_known(a.version)
    d = disasm.Disasm(rom, extra, data)
    d.trace_all()
    d.speculate_all()

    starts = [i for i in range(len(rom)) if cov[i] & 1]
    missed = [s for s in starts if s not in d.insns]
    confirmed = sum(1 for s in starts if s in d.insns)
    print('%d instruction starts ran; %d of the disassembler\'s %d '
          'instructions confirmed' % (len(starts), confirmed, len(d.insns)))
    print('missed: %d starts the disassembler doesn\'t have' % len(missed))
    shown = 0
    prev = -100
    for s in missed:
        if s - prev > 16 and shown < a.list:
            where = 'inside %06X' % d.owner[s] if s in d.owner else 'data'
            print('  %06X  %s' % (s, where))
            shown += 1
        prev = s
    rin = [i for i in range(len(rom)) if cov[i] & 0x10 and i in d.owner]
    print('read-in-code: %d bytes read as data inside code' % len(rin))
    shown = 0
    prev = -100
    for i in rin:
        if i - prev > 16 and shown < a.list:
            o = d.owner[i]
            print('  %06X  in %06X %s%s' % (
                i, o, d.insns[o].format(),
                '' if cov[o] & 1 else '  (never ran)'))
            shown += 1
        prev = i
    spec = [s for s in d.insns if not cov[s] & 1]
    print('unconfirmed: %d instructions never ran' % len(spec))
    if a.write:
        write_known(a.version, rom, cov, d)


def entries(rom, cov, d):
    """Executed starts that nothing executed reaches directly: no executed
    instruction falls through to them or branches or calls there.  These
    are the targets of indirect flow, what the static trace can't find."""
    import m68k
    reached = set()
    for s in range(len(rom)):
        if not cov[s] & 1:
            continue
        ins = d.insns.get(s) or m68k.decode(rom, s)
        if ins is None:
            continue
        if ins.flow not in ('jump', 'ret'):
            reached.add(s + ins.size)
        if ins.target is not None:
            reached.add(ins.target)
    return [s for s in range(len(rom)) if cov[s] & 1 and s not in reached]


def write_known(version, rom, cov, d):
    """code.VERSION.txt: the entries above that tracing from the vectors
    doesn't reach; data.VERSION.txt: runs of bytes read as data below the
    code area's end; longs.VERSION.txt: longs read as longs that hold a
    ROM address.  Both from the trace alone, so they can be
    regenerated whenever more is traced."""
    plain = disasm.Disasm(rom)
    plain.trace_all()
    code = [s for s in entries(rom, cov, d) if s not in plain.insns]
    with open(os.path.join(TOP, 'code.%s.txt' % version), 'w') as f:
        f.write('# instruction starts a runtime trace ran that nothing in '
                'the code reaches\n# directly (tools/coverage.py --write; '
                'tools/inputs/)\n')
        for s in code:
            f.write('%06X\n' % s)
    # bytes of instructions that ran are code even when also read (code
    # copied or scanned as data)
    import m68k
    ran = bytearray(len(rom))
    for s in range(d.code_limit):
        if cov[s] & 1:
            ins = m68k.decode(rom, s)
            for k in range(ins.size if ins else 2):
                ran[s + k] = 1
    runs = []
    k = 0
    while k < d.code_limit:
        if cov[k] & 0x10 and not ran[k]:
            j = k
            while j < d.code_limit and cov[j] & 0x10 and not ran[j]:
                j += 1
            runs.append((k, j))
            k = j
        else:
            k += 1
    with open(os.path.join(TOP, 'data.%s.txt' % version), 'w') as f:
        f.write('# ranges [lo, hi) of the code area a runtime trace read '
                'as data\n# (tools/coverage.py --write; tools/inputs/)\n')
        for lo, hi in runs:
            f.write('%06X %06X\n' % (lo, hi))
    # longs read whose value could be a ROM address: tools/disasm.py
    # makes the ones that fit pointers (Disasm.find_pointers)
    import struct
    longs = [k for k in range(disasm.HEADER_END, len(rom) - 3)
             if cov[k] & 0x08 and
             0x100 <= struct.unpack_from('>I', rom, k)[0] < len(rom)]
    with open(os.path.join(TOP, 'longs.%s.txt' % version), 'w') as f:
        f.write('# ROM longs a runtime trace read as longs whose value is '
                'in the ROM\n# (tools/coverage.py --write; '
                'tools/inputs/)\n')
        for k in longs:
            f.write('%06X\n' % k)
    print('wrote %d entries to code.%s.txt, %d ranges to data.%s.txt, '
          '%d to longs.%s.txt' % (len(code), version, len(runs), version,
                                  len(longs), version))

if __name__ == '__main__':
    main()
