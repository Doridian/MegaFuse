#!/usr/bin/env bash
# Run every pad script in tools/inputs/ headless (tools/mdtrace) on one
# version, then regenerate code.VERSION.txt and data.VERSION.txt from the
# combined coverage (tools/coverage.py --write).  Screenshots every 3000
# frames land next to the coverage in build/VERSION/trace/.
#
#   tools/trace_all.sh VERSION [FRAMES]
set -euo pipefail
cd "$(dirname "$0")/.."
V=${1:?usage: tools/trace_all.sh VERSION [FRAMES]}
FRAMES=${2:-}
OUT=build/$V/trace
rm -rf "$OUT"
mkdir -p "$OUT"
# the boot checksum (us and eu: add.w (a0)+,d0 at 0x7BE2) reads the whole ROM
IGNORE="--ignore-reads 7BE2"
for f in tools/inputs/*.txt; do
    n=$(basename "$f" .txt)
    frames=${FRAMES:-60000}
    case $n in stage-*) frames=${FRAMES:-20000} ;; esac
    tp=
    case $n in battle4) tp=--teamplayer ;; esac
    nice -n 10 tools/mdtrace $tp --frames "$frames" --input "$f" $IGNORE \
        --cover "$OUT/$n.cov" --shot "$OUT/$n-" --shot-every 3000 \
        "baserom.$V.bin" &
done
wait
python3 tools/coverage.py "$V" "$OUT"/*.cov --write --list 0
