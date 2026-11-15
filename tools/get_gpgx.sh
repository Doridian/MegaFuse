#!/usr/bin/env bash
# Build Genesis Plus GX's libretro core with its CPU hooks (HOOK_CPU) into
# tools/gpgx/, for tools/src/mdtrace.c: execution and memory-access traces,
# RAM dumps and screenshots of headless runs.  It is a development tool only:
# nothing of it goes into the port, and it is fetched, never vendored (its
# license forbids commercial use; see its LICENSE.txt).
#
# The core normally exports only retro_*; the hook setter, the register
# reader and two readers of the 68000's clock (below) are exported too, so
# mdtrace can dlsym them.  PC-relative reads go through the hook too
# (below).
#
#   tools/get_gpgx.sh
set -euo pipefail
REPO=https://github.com/libretro/Genesis-Plus-GX
COMMIT=b9c2316441978d4015ba30e9a95d9b0e6f8ee981
# bumped when the build below changes, so `make setup` rebuilds
BUILD=4
TOP=$(cd "$(dirname "$0")" && pwd)
DEST=$TOP/gpgx
SO=$DEST/genesis_plus_gx_libretro.so
[ -f "$SO" ] && [ "$(cat "$DEST/COMMIT" 2>/dev/null)" = "$COMMIT $BUILD" ] && exit 0

mkdir -p "$DEST"
SRC=$DEST/src
if [ ! -d "$SRC/.git" ]; then
    git init -q "$SRC"
    git -C "$SRC" remote add origin "$REPO"
fi
git -C "$SRC" fetch -q --depth 1 origin "$COMMIT"
git -C "$SRC" checkout -q --force FETCH_HEAD
git -C "$SRC" clean -qfdx
cat > "$SRC/libretro/link.T" <<'EOF'
{
   global: retro_*; set_cpu_hook; m68k_get_reg; mdtrace_m68k_cycles;
           mdtrace_m68k_refresh;
   local: *;
};
EOF
# The 68000's PC-relative reads (`move.l (tbl,pc,d0.w),a0`) don't go
# through the read hook in Musashi: route them through it, so tables read
# that way are traced like any other data
python3 - "$SRC/core/m68k/m68kcpu.h" <<'EOF'
import sys
p = sys.argv[1]
s = open(p).read()
old = """#define m68ki_read_pcrel_8(A) m68k_read_pcrelative_8(A)
#define m68ki_read_pcrel_16(A) m68k_read_pcrelative_16(A)
#define m68ki_read_pcrel_32(A) m68k_read_pcrelative_32(A)"""
new = "".join(
    "#define m68ki_read_pcrel_%d(A) ({ uint a_ = (A); "
    "uint v_ = m68k_read_pcrelative_%d(a_); if (cpu_hook) "
    "cpu_hook(HOOK_M68K_R, %d, a_, v_); v_; })\n" % (n, n, n // 8)
    for n in (8, 16, 32))
assert old in s
open(p, "w").write(s.replace(old, new))
EOF
# An interrupt enabled while pending comes one instruction late: the core
# runs that instruction itself, without the execution hook; call it, so
# traces (mdtrace --timeline) have every instruction
python3 - "$SRC/core/m68k/m68kcpu.c" <<'EOF'
import sys
p = sys.argv[1]
s = open(p).read()
old = "      irq_latency = 1;\n"
new = old + ("#ifdef HOOK_CPU\n      if (cpu_hook) cpu_hook(HOOK_M68K_E, 0, "
             "REG_PC, 0);\n#endif\n")
assert s.count(old) == 1
open(p, "w").write(s.replace(old, new))
EOF
# The 68000's clock (master clocks into the frame) and when its next bus
# refresh is due, for mdtrace --cycles
cat >> "$SRC/core/m68k/m68kcpu.c" <<'EOF'

unsigned int mdtrace_m68k_cycles(void) { return m68k.cycles; }
int mdtrace_m68k_refresh(void) { return m68k.refresh_cycles; }
EOF
# -fcommon: core/debug/cpuhook.h defines cpu_hook in a header (no extern),
# which GCC 10+ and clang reject as multiple definitions by default
CFLAGS=-fcommon make -s -C "$SRC" -f Makefile.libretro HOOK_CPU=1 -j"$(nproc)" >/dev/null
cp "$SRC/genesis_plus_gx_libretro.so" "$SO"
echo "$COMMIT $BUILD" > "$DEST/COMMIT"
echo "Genesis Plus GX ($COMMIT) with HOOK_CPU in $SO"
