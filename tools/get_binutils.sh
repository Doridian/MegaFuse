#!/usr/bin/env bash
# Build GNU binutils for m68k-elf (as, ld, objcopy, objdump, nm) into
# tools/binutils, from the GNU release tarball checked against a pinned
# sha256.  Distributions rarely package an m68k cross binutils, and the
# build only needs a host C compiler.  An m68k-elf- binutils already on the
# PATH works too: `make CROSS=m68k-elf-`.
#
#   tools/get_binutils.sh [TARBALL]     (TARBALL: a downloaded copy to use)
set -euo pipefail
VER=2.47
SUM=154ab23b60070e8f27013c22977f1129425d67d1e8acd6e13010e617811e4cff
TOP=$(cd "$(dirname "$0")" && pwd)
DEST=$TOP/binutils
[ -x "$DEST/bin/m68k-elf-as" ] && exit 0

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
TAR=${1:-$WORK/binutils-$VER.tar.xz}
[ -f "$TAR" ] || curl -fsSL -o "$TAR" "https://ftp.gnu.org/gnu/binutils/binutils-$VER.tar.xz"
echo "$SUM  $TAR" | sha256sum --check --quiet
tar xJf "$TAR" -C "$WORK"
mkdir "$WORK/build"
cd "$WORK/build"
"../binutils-$VER/configure" --target=m68k-elf --prefix="$DEST" \
    --disable-nls --disable-werror --disable-gdb --disable-gdbserver \
    --disable-sim --disable-gprofng --disable-multilib >/dev/null
make -j"$(nproc)" >/dev/null
make install >/dev/null
echo "binutils $VER (m68k-elf) in $DEST"
