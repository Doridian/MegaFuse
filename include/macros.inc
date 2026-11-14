| Included by every asm file (tools/disasm.py writes them).

| gas would otherwise turn `move.l #0,d0` into moveq and `add.w #1,d0` into
| addq: the ROM's encodings are written as they are (tools/test_m68k.py)
	.opt noquick
	.text
