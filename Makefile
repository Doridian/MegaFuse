# Mega Bomberman: the ROM rebuilt from asm/, sha1-checked.
#
#   make setup                 once: GNU binutils for m68k-elf in tools/binutils,
#                              Genesis Plus GX and tools/mdtrace (tracing)
#   make VERSION=us extract    baserom.us.bin -> asm/ (tools/disasm.py)
#   make VERSION=us            assemble, link, check
#   make test                  ROM-free self-tests (CI)
#
# VERSION is us or eu.  asm/ holds one version at a time (.version names
# it): `make clean` before switching.

BASENAME  = mbm
VERSION  ?= us
VERSIONS  = us eu
ifeq ($(filter $(VERSION),$(VERSIONS)),)
$(error Unknown VERSION '$(VERSION)'; expected one of $(VERSIONS))
endif

# asm/ holds one version; another version's leftovers would only show up
# as a sha1 mismatch
ifneq ($(filter-out clean distclean setup test extract,$(or $(MAKECMDGOALS),all)),)
TREE_VERSION := $(shell cat .version 2>/dev/null)
ifneq ($(TREE_VERSION),)
ifneq ($(TREE_VERSION),$(VERSION))
$(error asm/ holds $(TREE_VERSION); run 'make clean' first)
endif
endif
endif

PYTHON   ?= python3
HOSTCC   ?= cc
CROSS    ?= tools/binutils/bin/m68k-elf-
AS        = $(CROSS)as
LD        = $(CROSS)ld
OBJCOPY   = $(CROSS)objcopy
ASFLAGS   = -m68000 --register-prefix-optional -no-pad-sections -I include

BUILD_DIR = build/$(VERSION)
BASEROM   = baserom.$(VERSION).bin
TARGET    = $(BUILD_DIR)/$(BASENAME).$(VERSION)

S_FILES  := $(wildcard asm/*.s)
O_FILES  := $(patsubst asm/%.s,$(BUILD_DIR)/asm/%.o,$(S_FILES))

.PHONY: all extract clean distclean setup test check
.DEFAULT_GOAL := all

all: check

check: $(TARGET).bin
	@sha1sum --quiet --check $(BASENAME).$(VERSION).sha1 && \
	    echo "$(TARGET).bin: OK"

setup:
	tools/get_binutils.sh
	tools/get_gpgx.sh
	$(MAKE) tools/mdtrace

# the headless tracer over Genesis Plus GX's libretro core (tools/get_gpgx.sh)
LIBRETRO_H = tools/gpgx/src/libretro/libretro-common/include
tools/mdtrace: tools/src/mdtrace.c
	$(HOSTCC) -O2 -Wall -Wextra -I $(LIBRETRO_H) -o $@ $< -ldl -lz

extract: $(BASEROM)
	@sha1sum --quiet --check <(sed 's|build/$(VERSION)/$(BASENAME).$(VERSION).bin|$(BASEROM)|' $(BASENAME).$(VERSION).sha1) || \
	    { echo "$(BASEROM) is not the expected ROM ($(BASENAME).$(VERSION).sha1)"; exit 1; }
	@if [ -f .version ] && [ "$$(cat .version)" != "$(VERSION)" ]; then \
	    echo "asm/ holds $$(cat .version); run 'make clean' first"; exit 1; fi
	rm -rf asm
	$(PYTHON) tools/disasm.py $(VERSION)

SHELL = /bin/bash

$(BUILD_DIR)/asm/%.o: asm/%.s include/macros.inc
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) -o $@ $<

$(BUILD_DIR)/$(BASENAME).ld: mbm.ld asm/link.ld
	@mkdir -p $(dir $@)
	sed 's|BUILD_DIR|$(BUILD_DIR)|' asm/link.ld | grep -v '^/\*' > $@.objs
	sed -e '/^OBJECTS$$/r $@.objs' -e '/^OBJECTS$$/d' mbm.ld > $@

$(TARGET).elf: $(O_FILES) $(BUILD_DIR)/$(BASENAME).ld
	$(LD) -T $(BUILD_DIR)/$(BASENAME).ld -Map $(TARGET).map -o $@

$(TARGET).bin: $(TARGET).elf
	$(OBJCOPY) -O binary -j .text $< $@
	$(PYTHON) tools/checksum.py $@

test:
	$(PYTHON) tools/test_m68k.py --as $(AS)

clean:
	rm -rf asm build .version

distclean: clean
	rm -rf tools/binutils
