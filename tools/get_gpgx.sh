#!/usr/bin/env bash
# Build Genesis Plus GX's libretro core with its CPU hooks (HOOK_CPU) into
# tools/gpgx/, for tools/src/mdtrace.c: execution and memory-access traces,
# RAM dumps and screenshots of headless runs.  It is a development tool only:
# nothing of it goes into the port, and it is fetched, never vendored (its
# license forbids commercial use; see its LICENSE.txt).
#
# The core normally exports only retro_*; the hook setter and the register
# reader are exported too, so mdtrace can dlsym them.
#
#   tools/get_gpgx.sh
set -euo pipefail
REPO=https://github.com/libretro/Genesis-Plus-GX
COMMIT=b9c2316441978d4015ba30e9a95d9b0e6f8ee981
TOP=$(cd "$(dirname "$0")" && pwd)
DEST=$TOP/gpgx
SO=$DEST/genesis_plus_gx_libretro.so
[ -f "$SO" ] && [ "$(cat "$DEST/COMMIT" 2>/dev/null)" = "$COMMIT" ] && exit 0

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
   global: retro_*; set_cpu_hook; m68k_get_reg;
   local: *;
};
EOF
# -fcommon: core/debug/cpuhook.h defines cpu_hook in a header (no extern),
# which GCC 10+ and clang reject as multiple definitions by default
CFLAGS=-fcommon make -s -C "$SRC" -f Makefile.libretro HOOK_CPU=1 -j"$(nproc)" >/dev/null
cp "$SRC/genesis_plus_gx_libretro.so" "$SO"
echo "$COMMIT" > "$DEST/COMMIT"
echo "Genesis Plus GX ($COMMIT) with HOOK_CPU in $SO"
