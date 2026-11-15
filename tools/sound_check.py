#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy"]
# ///
"""The port's sound against the original's: same pad script, same sound?

    tools/sound_check.py [SCRIPT...] [--frames N] [--block N] [--window N]
                         [--sim X] [--db X]

Runs the port (target/release/megafuse, `cargo build --release`) and the
original in Genesis Plus GX (tools/mdtrace) on each pad script
(tools/inputs/, all by default; `none` for no input), both with --wav,
and compares the sound a block of --block frames at a time: the spectra
(energy in 63 bands, 60 Hz-8 kHz, a log scale; cosine similarity) and
the levels (RMS, in dB).  The port's time drifts from the console's
(docs/PORT.md, "Time"), so each block of the port's is compared with the
original's within --window frames of it and the closest taken.  Blocks
silent in both are skipped.  Reports, per script, the first block that
differs (similarity under --sim or levels more than --db apart) and the
mean similarity up to it.  The scripts start playing some 2400 frames
in, so the default, 6000, takes each into its game.  Past a few
thousand frames the game itself has drifted from the original (its
random numbers, docs/PORT.md "Time"): a difference there is the game's,
not the sound's; screenshots at that frame show which.

Needs uv (numpy comes with it).
"""
import argparse
import os
import subprocess
import sys
import tempfile
import wave

import numpy as np

TOP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# the exact build (make exact; docs/PORT.md, "Rewriting")
PORT = os.environ.get('MEGAFUSE', os.path.join(TOP, 'target/exact/release/megafuse'))
# NTSC: 53693175 / (3420 * 262)
FPS = 53693175 / (3420 * 262)
BANDS = np.geomspace(60, 8000, 64)
# RMS (of 32768) under which a block counts as silent
SILENT = 30


def load(path):
    w = wave.open(path)
    rate = w.getframerate()
    d = np.frombuffer(w.readframes(w.getnframes()), dtype='<i2')
    return d.reshape(-1, 2).astype(np.float64).mean(axis=1), rate


def frames(x, rate):
    """Per frame: the band energies (of two frames' worth, Hann-windowed,
    so they overlap) and the power (DC aside)."""
    n = int(round(rate / FPS))
    count = len(x) // n - 1
    f = np.fft.rfftfreq(2 * n, 1 / rate)
    which = np.digitize(f, BANDS) - 1
    keep = (which >= 0) & (which < len(BANDS) - 1)
    hann = np.hanning(2 * n)
    spec = np.zeros((count, len(BANDS) - 1))
    power = np.zeros(count)
    for k in range(count):
        seg = x[k * n:(k + 2) * n]
        m = np.abs(np.fft.rfft(seg * hann))
        spec[k] = np.bincount(which[keep], m[keep], len(BANDS) - 1)
        power[k] = np.var(seg[:n])
    return spec, power


def compare(port, orig, block, window, min_sim, max_db):
    """(last good frame, mean similarity, why the first bad block is)."""
    ps, pp = port
    os_, op = orig
    sims = []
    last = 0
    for f in range(0, len(ps) - block + 1, block):
        a = ps[f:f + block].sum(axis=0)
        ra = np.sqrt(pp[f:f + block].mean())
        best = None
        for s in range(-window, window + 1):
            g = f + s
            if g < 0 or g + block > len(os_):
                continue
            b = os_[g:g + block].sum(axis=0)
            rb = np.sqrt(op[g:g + block].mean())
            na, nb = np.linalg.norm(a), np.linalg.norm(b)
            sim = a @ b / (na * nb) if na and nb else float(na == nb)
            if best is None or sim > best[0]:
                best = (sim, s, rb)
        if best is None:
            break
        sim, s, rb = best
        if ra < SILENT and rb < SILENT:
            last = f + block
            continue
        db = 20 * np.log10(max(ra, 1) / max(rb, 1))
        if sim < min_sim or abs(db) > max_db:
            why = ('frames %d-%d: similarity %.3f, levels %.0f/%.0f '
                   '(%+.1f dB), best at %+d frames'
                   % (f, f + block, sim, ra, rb, db, s))
            return last, np.mean(sims) if sims else 0, why
        sims.append(sim)
        last = f + block
    return last, np.mean(sims) if sims else 0, None


def check(script, n, tmp, a):
    name = os.path.basename(script)[:-4] if script else 'none'
    port = os.path.join(tmp, name + '-port.wav')
    orig = os.path.join(tmp, name + '-orig.wav')
    cmd = [PORT, 'baserom.us.bin',
           '--frames', str(n), '--every', '0', '--wav', port]
    mt = ['nice', os.path.join(TOP, 'tools/mdtrace'), '--frames',
          str(n + a.window), '--wav', orig, 'baserom.us.bin']
    if script:
        cmd[2:2] = ['--input', script]
        mt[4:4] = ['--input', script]
        text = open(script).read()
        if '# teamplayer\n' in text:
            mt.insert(4, '--teamplayer')
        if '# wayplay\n' in text:
            mt.insert(4, '--wayplay')
    r = subprocess.run(['nice'] + cmd, cwd=TOP, capture_output=True,
                       text=True)
    subprocess.run(mt, cwd=TOP, check=True, capture_output=True)
    if r.returncode != 0 or not os.path.exists(port):
        return name, 0, 0, 'the port stopped: %s' % ' '.join(
            r.stderr.strip().splitlines()[-2:])
    p = frames(*load(port))
    o = frames(*load(orig))
    last, sim, why = compare(p, o, a.block, a.window, a.sim, a.db)
    return name, last, sim, why


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('scripts', nargs='*')
    ap.add_argument('--frames', type=int, default=6000)
    ap.add_argument('--block', type=int, default=30)
    ap.add_argument('--window', type=int, default=8)
    ap.add_argument('--sim', type=float, default=0.95)
    ap.add_argument('--db', type=float, default=3.0)
    a = ap.parse_args()
    scripts = a.scripts or sorted(
        os.path.join(TOP, 'tools/inputs', x)
        for x in os.listdir(os.path.join(TOP, 'tools/inputs'))
        if x.endswith('.txt'))
    scripts = [None if x == 'none' else x for x in scripts]
    tmp = tempfile.mkdtemp(prefix='sound-', dir=os.path.join(TOP, 'build'))
    bad = 0
    for sc in scripts:
        name, last, sim, why = check(sc, a.frames, tmp, a)
        if why:
            bad += 1
            print('%-16s same to %d (similarity %.3f), then %s'
                  % (name, last, sim, why))
        else:
            print('%-16s ok to %d (similarity %.3f)' % (name, last, sim))
    print('kept in %s' % tmp)
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
