#!/usr/bin/env python3
"""Run the shifted build against the original and compare what they do.

    tools/shift_check.py VERSION [--frames N] [--every N] [--input NAME...]
                                 [--first]

The shifted ROM (make SHIFT=1: build/VERSION-shift/mbm.VERSION.bin) has
padding after each object and before most functions (Makefile, "SHIFT"),
so everything in the code moves, a few bytes at a time.  A ROM address
the disassembly still writes as a number (tools/disasm.py) now points at
the wrong thing.  Both ROMs run headless (tools/mdtrace) with the same pad
scripts (tools/inputs/, all by default), and every --every frames their
screenshots must be identical and their work RAM the same, but for ROM
addresses: a long (or a word, an address below 0x10000) that in the
original points at `label + off` must, in the shifted build, point at
that label's new address + off.  The map from the original to the
shifted build pairs the two ELFs' symbol tables by name.

--first narrows down where a run first goes wrong: RAM every frame up to
the first checkpoint that differs, and the first frame whose RAM differs,
with the bytes that do.

The stack, down to the lowest the stack pointer has been (mdtrace's .sp
files), isn't compared: it holds return addresses, saved registers and
what earlier calls left there, pieces of addresses among them (a byte
pushed over the low byte of an old return address).  It is control state:
where it goes wrong, the screens or the rest of RAM do.

The bosses' palette cycle (func_00AA4C) reads its colours from code
(0x19068 in eu: `jsr (0x19144).l` among them), so a shifted build shows
other colours there however symbolic it is: the stage-*-boss scripts are
compared only up to the fight (LIMITS).

The 68000 has no cache and the Mega Drive's ROM no wait states, so moving
code doesn't change its timing: the runs should agree exactly.
"""
import argparse
import bisect
import os
import re
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

TOP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
NM = os.path.join(TOP, 'tools/binutils/bin/m68k-elf-nm')
LABEL = re.compile(r'^(?:func|loc|D|jtbl)_([0-9A-F]{6})(?:_[a-z]+)?$')


def symbols(elf):
    """The labels' addresses in an ELF (by name: eu's are named after
    their address in us)."""
    out = {}
    text = subprocess.run([NM, elf], capture_output=True, text=True,
                          check=True).stdout
    for line in text.splitlines():
        t = line.split()
        if len(t) == 3 and LABEL.match(t[2]):
            out[t[2]] = int(t[0], 16)
    return out


class AddrMap:
    """Original ROM address -> shifted, through the nearest label at or
    before it."""

    def __init__(self, orig_elf, elf, rom_len):
        self.pairs = set()     # see PAST_TABLE
        old, new = symbols(orig_elf), symbols(elf)
        pairs = {0: 0}
        for name, a in old.items():
            if name in new:
                pairs[a] = new[name]
        self.orig = sorted(pairs)
        self.new = [pairs[a] for a in self.orig]
        self.rom_len = rom_len

    def __call__(self, a):
        i = bisect.bisect_right(self.orig, a) - 1
        return self.new[i] + a - self.orig[i]

    def back(self, a):
        """Shifted -> original (an address in a pad maps into the label
        after it, past its start)."""
        i = bisect.bisect_right(self.new, a) - 1
        return self.orig[i] + a - self.new[i]

    def moved(self, a):
        return 0x200 <= a < self.rom_len and self(a) != a


def run(rom, script, frames, every, out, teamplayer):
    cmd = ['nice', '-n', '10', os.path.join(TOP, 'tools/mdtrace'),
           '--frames', str(frames), '--shot', out, '--ram', out,
           '--shot-every', str(every), rom]
    if script:
        cmd[4:4] = ['--input', script]
    if teamplayer:
        cmd.insert(4, teamplayer)
    subprocess.run(cmd, check=True, cwd=TOP)


# ROM bytes read past the end of a table: what follows the table, which a
# shifted build moves, so the values differ (and get copied around: saved
# registers on the stack).  (table's address in the original, offset into
# it, size): the original's bytes there and the shifted build's, as a pair,
# are the same value wherever they turn up in RAM.
PAST_TABLE = [
    # func_0005C0 calls func_000B5A with slot 0xFF (none): a5 is
    # D_000B7E[0xFF], code, stored at 0xFF8A5A and saved by movem
    (0xB7E, 0x3FC, 4),
]


def past_table_pairs(amap, roms):
    out = set()
    for table, k, size in PAST_TABLE:
        a = roms[0][table + k:table + k + size]
        b = roms[1][amap(table) + k:amap(table) + k + size]
        out.add((a, b))
        # and its halves: a word of it lingers where the long was
        for h in range(0, size, 2):
            out.add((a[h:h + 2], b[h:h + 2]))
    return out


STACK_TOP = 0xFF00   # the reset vector's stack pointer, 0xFFFF00


def ram_diff(a, b, amap, dead=(0, 0)):
    """Byte offsets of work RAM that differ once ROM addresses are
    translated, but for the stack (offsets dead[0] to dead[1]-1)."""
    bad = []
    k = 0
    n = len(a)
    pairs = amap.pairs
    while k < n:
        if a[k] == b[k] or dead[0] <= k < dead[1]:
            k += 1
            continue
        # a long at k-3..k, even, that is a moved ROM address
        ok = False
        for s in range(k & ~1, max(k - 4, -1), -2):
            if s + 4 <= n:
                va = int.from_bytes(a[s:s + 4], 'big')
                vb = int.from_bytes(b[s:s + 4], 'big')
                if (amap.moved(va) and amap(va) == vb) or \
                        (a[s:s + 4], b[s:s + 4]) in pairs:
                    k = s + 4
                    ok = True
                    break
        if not ok:
            s = k & ~1
            va = int.from_bytes(a[s:s + 2], 'big')
            vb = int.from_bytes(b[s:s + 2], 'big')
            # a word: an address in the first 64 KB, or the low half of
            # one whose high half was overwritten (0xFF80C8 holds a long
            # address, then a word in its high half: another variable)
            if any(amap.moved(h << 16 | va) and
                   amap(h << 16 | va) & 0xFFFF == vb for h in range(16)) or \
                    (a[s:s + 2], b[s:s + 2]) in pairs:
                k = s + 2
                ok = True
        s = k & ~1
        if not ok and s + 4 <= n and a[s] == b[s] == 0 and \
                a[s + 1] < 0x10 and b[s + 1] < 0x10 and \
                a[s + 2:s + 4] == b[s + 2:s + 4]:
            # the stale high word of a ROM address whose low word was
            # overwritten (FF8A5E: a pointer, then a word stored in its low
            # half): the high words are 0x0000-0x000F, the low ones equal
            k = s + 2
            ok = True
        if not ok:
            bad.append(k)
            k += 1
    return bad


# scripts compared only up to a frame, and why
LIMITS = {
    # the boss's palette cycle reads its colours from code (above)
    'stage-1-boss': 4800, 'stage-2-boss': 4800, 'stage-3-boss': 4800,
    'stage-4-boss': 4800, 'stage-5-boss': 4500,
}


def teamplayer(path):
    """mdtrace's option for the script's multitap: `# teamplayer` (Sega's
    Team Player) or `# wayplay` (EA's 4-Way Play), or None."""
    text = open(path).read()
    if '# teamplayer\n' in text:
        return '--teamplayer'
    if '# wayplay\n' in text:
        return '--wayplay'
    return None


def script_compare(path):
    """The frame a script is compared up to, its `# compare N` line
    (tools/gen_inputs.py: an area's last stage's exit goes on into the
    boss stage, whose palette cycle reads code), or None."""
    for line in open(path):
        if line.startswith('# compare '):
            return int(line.split()[2])
    return None


def script_frames(path):
    """A script's own length, its `# frames N` line (tools/gen_inputs.py
    writes one where a script runs for longer than the default: the
    ending's credits end at about 22000, the exits at up to 11500)."""
    for line in open(path):
        if line.startswith('# frames '):
            return int(line.split()[2])
    return 0


def checkpoints(frames, every):
    out = list(range(every, frames + 1, every))
    if not out or out[-1] != frames:
        out.append(frames)
    return out


def compare(tmp, name, f, amap):
    """(png same, unexplained RAM offsets) at frame f."""
    p = [os.path.join(tmp, '%s-%s-%06d' % (name, w, f))
         for w in ('orig', 'shift')]
    png = open(p[0] + '.png', 'rb').read() == open(p[1] + '.png',
                                                    'rb').read()
    ra, rb = (open(x + '.bin', 'rb').read() for x in p)
    # the stack, down to the lowest it has been: return addresses and
    # saved registers, and pieces of them left by byte pushes and calls
    sp = [[int(x, 16) & 0xFFFF for x in open(q + '.sp').read().split()]
          for q in p]
    dead = (min(sp[0][1], sp[1][1]), STACK_TOP)
    return png, ram_diff(ra, rb, amap, dead)


def describe(bad, ra, rb, limit=12):
    out = []
    for k in bad[:limit]:
        out.append('FF%04X %02X/%02X' % (k, ra[k], rb[k]))
    return ' '.join(out) + (' ...' if len(bad) > limit else '')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('version')
    ap.add_argument('--frames', type=int, default=3000)
    ap.add_argument('--every', type=int, default=300)
    ap.add_argument('--input', nargs='*',
                    help='scripts in tools/inputs/ (names), default all')
    ap.add_argument('--first', action='store_true',
                    help='find the first frame each script goes wrong')
    ap.add_argument('--pcs', metavar='NAME:FRAME',
                    help='the first instruction the runs differ at in '
                    'FRAME-1..FRAME of script NAME')
    a = ap.parse_args()
    if a.pcs:
        name, frame = a.pcs.split(':')
        v = a.version
        amap = AddrMap(os.path.join(TOP, 'build', v, 'mbm.%s.elf' % v),
                       os.path.join(TOP, 'build', v + '-shift',
                                    'mbm.%s.elf' % v), os.path.getsize(
            os.path.join(TOP, 'build', v, 'mbm.%s.bin' % v)))
        amap.pairs = past_table_pairs(amap, tuple(open(os.path.join(
            TOP, 'build', v + w, 'mbm.%s.bin' % v), 'rb').read()
            for w in ('', '-shift')))
        tmp = tempfile.mkdtemp(prefix='pcs-', dir=os.path.join(
            TOP, 'build', v + '-shift'))
        pcs_diff(a, amap, tmp, name, int(frame))
        return 0
    v = a.version
    orig = os.path.join(TOP, 'build', v, 'mbm.%s.bin' % v)
    shift = os.path.join(TOP, 'build', v + '-shift', 'mbm.%s.bin' % v)
    elf = os.path.join(TOP, 'build', v + '-shift', 'mbm.%s.elf' % v)
    amap = AddrMap(orig[:-4] + '.elf', elf, os.path.getsize(orig))
    amap.pairs = past_table_pairs(amap, (open(orig, 'rb').read(),
                                         open(shift, 'rb').read()))
    names = a.input or sorted(
        x[:-4] for x in os.listdir(os.path.join(TOP, 'tools/inputs'))
        if x.endswith('.txt'))
    tmp = tempfile.mkdtemp(prefix='shift-', dir=os.path.join(
        TOP, 'build', v + '-shift'))

    def one(name):
        script = os.path.join(TOP, 'tools/inputs', name + '.txt')
        tp = teamplayer(script)
        frames = max(a.frames, script_frames(script))
        for which, rom in (('orig', orig), ('shift', shift)):
            run(rom, script, frames, a.every,
                os.path.join(tmp, '%s-%s-' % (name, which)), tp)
        limit = script_compare(script) or LIMITS.get(name)
        for f in checkpoints(frames, a.every):
            if limit is not None and f > limit:
                break
            png, bad = compare(tmp, name, f, amap)
            if not png or bad:
                return name, f, png, bad
        return name, None, True, []

    failed = []
    with ThreadPoolExecutor(os.cpu_count()) as ex:
        for name, f, png, bad in ex.map(one, names):
            if f is None:
                print('%-16s ok' % name)
                continue
            failed.append((name, f))
            print('%-16s frame %d: %s, %d RAM bytes' % (
                name, f, 'screen same' if png else 'screen differs',
                len(bad)))
    if a.first and failed:
        def first(item):
            name, f = item
            script = os.path.join(TOP, 'tools/inputs', name + '.txt')
            lo = max(0, f - a.every)
            for which, rom in (('orig', orig), ('shift', shift)):
                run(rom, script, f, 1,
                    os.path.join(tmp, '%s-1-%s-' % (name, which)),
                    teamplayer(script))
            for g in range(max(1, lo), f + 1):
                png, bad = compare(tmp, name + '-1', g, amap)
                if bad or not png:
                    p = os.path.join(tmp, '%s-1-%%s-%06d.bin' % (name, g))
                    ra = open(p % 'orig', 'rb').read()
                    rb = open(p % 'shift', 'rb').read()
                    return '%-16s first at frame %d%s: %s' % (
                        name, g, '' if png else ' (screen)',
                        describe(bad, ra, rb))
            return '%-16s nothing found before %d' % (name, f)
        with ThreadPoolExecutor(os.cpu_count()) as ex:
            for line in ex.map(first, failed):
                print(line)
    print('kept in %s' % tmp)
    return 1 if failed else 0


def pcs_diff(a, amap, tmp, name, frame):
    """Run frame-1 and frame (the one whose RAM first differs) in both
    ROMs with every PC and ROM read logged, and print the first
    instruction where the shifted run, mapped back, does something else,
    or the first ROM read of another place or value."""
    v = a.version
    script = os.path.join(TOP, 'tools/inputs', name + '.txt')
    logs = []
    for w in ('', '-shift'):
        rom = os.path.join(TOP, 'build', v + w, 'mbm.%s.bin' % v)
        out = os.path.join(tmp, '%s%s.pcs' % (name, w))
        cmd = [os.path.join(TOP, 'tools/mdtrace'), '--frames',
               str(frame + 1), '--input', script, '--pcs', out,
               '--pcs-frames', '%d:%d' % (max(0, frame - 1), frame + 1),
               rom]
        if teamplayer(script):
            cmd.insert(1, teamplayer(script))
        subprocess.run(cmd, check=True, cwd=TOP)
        logs.append(out)
    with open(logs[0]) as fa, open(logs[1]) as fb:
        prev = []
        for n, (la, lb) in enumerate(zip(fa, fb)):
            ta, tb = la.split(), lb.split()
            pa, pb = int(ta[1], 16), int(tb[1], 16)
            back = amap.back(pb) if pb < amap.rom_len else pb
            same = ta[2:] == tb[2:]
            if not same and len(ta) > 2 and ta[2] == tb[2]:
                # a ROM address read from the ROM (a vector, a pointer
                # table): the shifted one is the translated one
                va, vb = int(ta[3], 16), int(tb[3], 16)
                same = (amap.moved(va) and amap(va) == vb) or (
                    bytes.fromhex(ta[3]), bytes.fromhex(tb[3])) in amap.pairs
            if len(ta) > 2 and ta[2:] != tb[2:] and (
                    bytes.fromhex(ta[3]),
                    bytes.fromhex(tb[3])) in amap.pairs:
                continue         # PAST_TABLE: its own address too
            if pa != back or not same:
                what = 'read' if len(ta) > 2 else 'instruction'
                print('%s: %s %d (frame %s): original %06X %s, '
                      'shifted %06X %s (= original %06X); after: %s' % (
                          name, what, n, ta[0], pa, ' '.join(ta[2:]), pb,
                          ' '.join(tb[2:]), back,
                          ' '.join('%06X' % p for p in prev[-12:])))
                return
            if len(ta) == 2:
                prev.append(pa)
    print('%s: no difference in the PCs' % name)


if __name__ == '__main__':
    sys.exit(main())
