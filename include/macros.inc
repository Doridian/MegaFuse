| Included by every asm file (tools/disasm.py writes them).

| gas would otherwise turn `move.l #0,d0` into moveq and `add.w #1,d0` into
| addq: the ROM's encodings are written as they are (tools/test_m68k.py)
	.opt noquick
	.text

| The version (make passes --defsym VERSION_EU=0 or 1): the source builds
| us and eu, `.if VERSION_EU` where they differ (tools/disasm.py)
	.ifndef VERSION_EU
	.set VERSION_EU, 0
	.endif

| SHIFT build (make SHIFT=1 passes --defsym SHIFT=1): shift_pad, which
| tools/disasm.py writes before functions nothing falls through into, is
| two zero bytes (SHIFT_FUNC_PAD there); in the matching build, nothing
	.ifndef SHIFT
	.set SHIFT, 0
	.endif
	.macro shift_pad
	.if SHIFT
	.space 2
	.endif
	.endm
