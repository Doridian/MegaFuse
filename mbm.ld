/* The ROM: every object's .text in ROM order (asm/link.ld, written by
   tools/disasm.py), from address 0.  The Makefile fills in OBJECTS. */
OUTPUT_ARCH(m68k)
ENTRY(rom_start)
SECTIONS
{
    . = 0;
    .text : SUBALIGN(2) {
        rom_start = .;
OBJECTS
        rom_end = .;
    }
    /DISCARD/ : { *(.data) *(.bss) *(.comment) *(.note*) }
}
