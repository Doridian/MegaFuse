#!/usr/bin/env python3
"""Find, by running the game, how to take each normal stage to its exit
with RAM writes, and write the plans to tools/exits.json (read by
tools/gen_inputs.py, which turns them into tools/inputs/exit-*.txt).

    tools/find_exits.py [STAGE...]     (us; every stage but the bosses and
                                        the final area, which
                                        tools/gen_inputs.py's ending covers)

Random play never clears a stage, so the stages' exits, the bonus stage
and the move to the next stage never ran in a trace.  This plays each
stage from its password with tools/mdtrace, a step at a time, reading
work RAM between runs (every run starts from power-on: they repeat):

  - Start on the stage card (0x9540 = 0xC0 with the player's slot,
    0xA432, in use), 120 frames after it shows (it ignores Start before),
    and on a screen that stays the same for 300 frames (the area map);
  - in play (0x9540 bits 4, 6 and 7 clear; bit 4 is the bonus stage
    after a gate): energy 0 (+0x11) for the object
    slots 1-19, the room's enemies and targets, as tools/gen_inputs.py's
    ending does; with the targets gone the gate opens (0x9540 bit 5);
  - the gate is either an object whose routine is func_00C1F6 (a task,
    its position at +0xE/+0x10; it checks the player with func_0018C4)
    or cells of the grid that become 4 when it opens (func_00C822's door
    to the next room).  A position (x, y) is in the cell at 0xFF989C +
    row[y >> 4] + (x >> 4), the rows' offsets being the longs at 0xFFA31A
    (func_001652): 16 or 32 bytes a row, as wide as the stage;
  - the player is put next to it, on a cell made floor (6), walking in;
  - until the stage (0x953C-0x953D, as in play) moves on, then 1500
    frames more: the bonus stage and the next stage's card.

A plan is the list of steps with their frames; every write is a number,
so a SHIFT build runs the scripts too.  The frames are us's.
"""
import json
import os
import shutil
import struct
import subprocess
import sys
import tempfile

TOP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(TOP, 'tools'))
import gen_inputs as g  # noqa: E402

OUT = os.path.join(TOP, 'tools', 'exits.json')
EVERY = 10                  # frames between RAM dumps
WAIT = 2000                 # frames after the energy writes the gate may
                            # take to open (polled every 50)


def cell_xy(c, r):
    return 0x28 + 16 * c, 0x18 + 16 * r


class Run:
    def __init__(self, code, plan, frames):
        self.dir = tempfile.mkdtemp(prefix='exits-', dir=os.path.join(
            TOP, 'build'))
        path = os.path.join(self.dir, 'in.txt')
        g.HERE = self.dir
        g.write('in.txt', g.exit_events(code, plan), 'find_exits')
        subprocess.run(['nice', os.path.join(TOP, 'tools/mdtrace'),
                        '--frames', str(frames), '--input', path,
                        '--ram', os.path.join(self.dir, 'r'),
                        '--shot-every', str(EVERY),
                        os.path.join(TOP, 'baserom.us.bin')],
                       check=True, cwd=TOP)
        self.frames = frames

    def ram(self, f):
        f = f - f % EVERY
        return open(os.path.join(self.dir, 'r%06d.bin' % f), 'rb').read()

    def done(self):
        shutil.rmtree(self.dir)


def state(r):
    if r[0x9540] == 0xC0 and r[0xA432] == 1:
        return 'card'
    if not r[0x9540] & 0xD0 and r[0xA432] == 1:
        return 'play'             # (bit 4: the bonus stage after a gate)
    return 'other'


def still(r):
    return r[0x9540], r[0xA432], r[0x953A:0x9540]


GRID0, ROWS = 0x989C, 0xA31A


def geometry(r):
    """The grid's row offsets (the longs at 0xFFA31A, func_001652: a cell
    is 0xFF989C + row[y >> 4] + (x >> 4)) and its stride."""
    rows = []
    for k in range(64):
        v = struct.unpack_from('>I', r, ROWS + 4 * k)[0]
        if v > 0x800 or (rows and v <= rows[-1]):
            break
        rows.append(v)
    return rows, (rows[1] - rows[0] if len(rows) > 1 else 16)


def cell_of(rows, stride, off):
    for k, v in enumerate(rows):
        if v <= off < v + stride:
            return off - v, k
    return None


def gate_task(r):
    """func_00C1F6's task: its (x, y)."""
    for a in range(0x8000, 0x8A00, 2):
        if r[a:a + 4] == b'\x00\x00\xc1\xf6' and r[a + 4:a + 6] == b'\xff\xff':
            b = a - 4
            return struct.unpack('>HH', r[b + 0xE:b + 0x12])
    return None


def walk_into(before, r):
    """A 'walk' step into the gate, or None: the player on a cell next to
    it, made floor, walking in."""
    rows, stride = geometry(r)
    t = gate_task(r)
    if t is not None:
        c, rw = t[0] >> 4, (t[1] >> 4) + 2
        return dict(x=16 * c + 8, y=16 * rw + 8,
                    floor=GRID0 + rows[rw] + c, dir='U')
    size = rows[-1] + stride
    g0 = before[GRID0:GRID0 + size]
    g1 = r[GRID0:GRID0 + size]
    doors = [i for i in range(size) if g1[i] == 4 and g0[i] != 4]
    for i in doors:
        cr = cell_of(rows, stride, i)
        if cr is None:
            continue
        c, rw = cr
        for dc, dr, d in ((0, 1, 'U'), (0, -1, 'D'), (1, 0, 'L'),
                          (-1, 0, 'R')):
            n = (c + dc, rw + dr)
            if not (0 <= n[0] < stride and 0 <= n[1] < len(rows)):
                continue
            k = rows[n[1]] + n[0]
            if g1[k] == 4 or k in doors:
                continue
            return dict(x=16 * n[0] + 8, y=16 * n[1] + 8, floor=GRID0 + k,
                        dir=d)
    return None


def solve(name, code, limit=40000):
    plan = []
    f = 4500
    rooms = 0
    key = None                  # 0x953C-0x953D in play: which stage
    starts = 0                  # Starts in a row
    while f < limit:
        run = Run(code, plan, min(f + 3000, limit))
        try:
            nxt = None
            for t in range(f + EVERY, run.frames, EVERY):
                r = run.ram(t)
                if key is not None and r[0x953C:0x953E] != key:
                    print('%s: cleared at %d (%d rooms)' % (name, t, rooms))
                    return plan, t
                st = state(r)
                if st == 'card':
                    # it takes Start some 100 frames after it shows
                    nxt = dict(do='start', frame=t + 120)
                    break
                if st == 'other' and t - 300 > f and all(
                        still(run.ram(u)) == still(r)
                        for u in range(t - 300, t, 50)):
                    # a screen waiting for Start (the area map)
                    nxt = dict(do='start', frame=t + 10)
                    break
                if st == 'play' and state(run.ram(t - 30)) == 'play':
                    if key is None:
                        key = r[0x953C:0x953E]
                    nxt = 'play'
                    break
            if nxt is None:
                print('%s: stuck after %d' % (name, f))
                return plan, None
            if nxt != 'play':
                starts += 1
                if starts > 5:
                    # Start leads nowhere (a boss stage pauses on it)
                    print('%s: Start after Start at %d' % (name, t))
                    return plan, None
                plan.append(nxt)
                f = nxt['frame'] + 20
                continue
        finally:
            run.done()
        starts = 0
        # in play at t: kill, then look for the gate as it opens
        plan.append(dict(do='kill', frame=t))
        run = Run(code, plan, t + WAIT + 10)
        try:
            before = run.ram(t)
            w = None
            for u in range(t + 100, t + WAIT, 50):
                r = run.ram(u)
                if r[0x953C:0x953E] != key:
                    print('%s: cleared at %d (%d rooms)' % (name, u, rooms))
                    return plan, u
                if r[0x9540] & 0x20:
                    w = walk_into(before, r)
                    if w is not None:
                        break
        finally:
            run.done()
        if w is None:
            print('%s: no gate by %d (0x9540 %02X)' % (
                name, t + WAIT, r[0x9540]))
            return plan, None
        w.update(do='walk', frame=u)
        plan.append(w)
        rooms += 1
        f = u + 200
    return plan, None


def main():
    names = sys.argv[1:]
    plans = json.load(open(OUT)) if os.path.exists(OUT) else {}
    for name, code in g.PASSWORDS:
        if 'boss' in name or name == 'final' or (
                names and name not in names):
            continue
        plan, end = solve(name, code)
        if end is None:
            continue
        plans[name] = dict(code=code, plan=plan, end=end + 1500)
        with open(OUT, 'w') as fo:
            json.dump(plans, fo, indent=1, sort_keys=True)
            fo.write('\n')


if __name__ == '__main__':
    main()
