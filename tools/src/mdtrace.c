/*
 * mdtrace: run a Mega Drive ROM headless in Genesis Plus GX's libretro core
 * (built with HOOK_CPU by tools/get_gpgx.sh) and record what the 68000 does
 * with the ROM.
 *
 *   mdtrace [options] ROM
 *     --core PATH        the core (default tools/gpgx/genesis_plus_gx_libretro.so)
 *     --frames N         frames to run (default 3600)
 *     --input FILE       pad script (below)
 *     --cover FILE       coverage map: one byte per ROM byte, OR'd into an
 *                        existing file of the right size (so runs add up)
 *     --shot PREFIX      screenshots PREFIX<frame>.png, at --shot-every N
 *                        frames (default 600) and the last frame
 *     --wav FILE         the sound, a 16-bit stereo WAV at the core's rate
 *     --cycles FILE      what each instruction took, in master clocks (7 a
 *                        68000 cycle), the bus refresh at its fetch (14)
 *                        left out: lines "PC COST S COUNT", S 1 when it
 *                        skipped a refresh (long instructions); an
 *                        instruction an exception follows is left out
 *     --timeline FILE    the 68000's time: per frame, "FRAME N HASH", the
 *                        number of instructions and a hash of their
 *                        (PC, clock) in order; the clock counts master
 *                        clocks from power-on (896040 a frame)
 *     --timeline-at F    and frame F in full: "PC CLOCK" an instruction
 *     --vram PREFIX      VRAM, VSRAM, the sprite table's copy (big-endian)
 *                        and the registers as PREFIX<frame>.vram/.vsram/
 *                        .sat/.regs, the frames --shot takes
 *     --hashes FILE      each frame: "FRAME RAM SCREEN", the CRC-32s of work
 *                        RAM to the stack (0xFE00, big-endian) and of the
 *                        picture (RGB, its width by its height)
 *     --ztimeline FILE   the same for the Z80 (its PC, the master clock;
 *                        in full, the YM2612's timer B counter too)
 *     --zcover FILE      the Z80's coverage: 8 KB, one byte per address of
 *                        its RAM, 1 where an instruction started
 *     --ram PREFIX       work RAM (64 KB, big-endian) as PREFIX<frame>.bin,
 *                        same frames, and PREFIX<frame>.sp: the stack
 *                        pointer and the lowest it has been, in hex
 *     --region R         auto (default), ntsc-u, pal, ntsc-j
 *     --teamplayer       Sega's Team Player on port 1 (pads 1-4 on it)
 *     --wayplay          EA's 4-Way Play on both ports (pads 1-4 on it)
 *     --log FILE         every frame's pad state and the RAM hash
 *     --ignore-reads PC  don't record the reads the instruction at PC makes
 *                        (repeatable; the boot checksum reads the whole ROM)
 *     --watch ADDR       print (stderr) each read or write of the byte at ADDR
 *                        (hex, ROM or 0xFFxxxx; repeatable), with the frame
 *                        and the instruction's address, up to --watch-limit
 *                        (default 64) in all
 *     --pcs FILE         every instruction executed, `FRAME PC` per line,
 *                        and every read of the ROM, `FRAME ADDR Rn VALUE`,
 *     --pcs-frames A:B   in frames A to B-1 (default 0:1)
 *     --snap ADDR        each call of the routine at ADDR (hex): the
 *                        registers and work RAM when it is entered and when
 *                        it returns, PREFIX<n>-in.bin and PREFIX<n>-out.bin
 *                        (--snap-out PREFIX, up to --snap-limit N calls,
 *                        default 50): D0-D7 A0-A7 PC SR as big-endian longs,
 *                        then the 64 KB of work RAM (tools/routine_check.py)
 *     --snaps FILE       the same for each routine FILE lists (hex, one a
 *                        line), PREFIX<ADDR>-<n>-in.bin and -out.bin, up to
 *                        --snap-limit calls each (tools/rewrite_check.py)
 *     --edges FILE       every jump between instructions that ran (from the
 *                        start of one to the next that ran, where that isn't
 *                        2-10 bytes on), `FROM TO` once per pair: the
 *                        targets of indirect calls (tools/recomp)
 *
 * Coverage flags (tools/coverage.py reads them):
 *     0x01  an instruction started here
 *     0x02  read as a byte     0x04  read as a word (here and +1)
 *     0x08  read as a long (here to +3)
 *     0x10  read (any width, every byte read)
 *     0x20  written (a ROM write: mapper or a bug)
 *
 * Pad script: one line per change, `FRAME [PAD:]BUTTONS`, held from FRAME on
 * until that pad's next line.  PAD is 1-4 (default 1).  BUTTONS are letters
 * from UDLRABCSXYZM (S Start, M Mode) or `-` for none.  `#` starts a comment.
 * `FRAME =ADDR VALUE` writes VALUE (hex: 2, 4 or 8 digits, a byte, word or
 * long) to work RAM at ADDR (hex, 0xFF0000-0xFFFFFF) before FRAME runs: a
 * shortcut past what a script can't play (a boss's energy, a stage number).
 */
#include <dlfcn.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <zlib.h>

#include "libretro.h"

#define MAX_EVENTS 65536
#define ROM_MAX 0x400000

static const char *region = "auto";
static int teamplayer;
static int wayplay;

/* the core's functions */
static void (*p_retro_init)(void);
static void (*p_retro_deinit)(void);
static void (*p_set_env)(retro_environment_t);
static void (*p_set_video)(retro_video_refresh_t);
static void (*p_set_audio)(retro_audio_sample_t);
static void (*p_set_audio_batch)(retro_audio_sample_batch_t);
static void (*p_set_poll)(retro_input_poll_t);
static void (*p_set_state)(retro_input_state_t);
static bool (*p_load_game)(const struct retro_game_info *);
static void (*p_run)(void);
static void (*p_get_av_info)(struct retro_system_av_info *);
static unsigned (*p_m68k_cycles)(void);
static int (*p_m68k_refresh)(void);
static unsigned (*p_z80_cycles)(void);
static int (*p_ym_timer_b)(void);
static void *(*p_get_memory_data)(unsigned);
static size_t (*p_get_memory_size)(unsigned);
static void (*p_set_port)(unsigned, unsigned);
static void (*p_set_cpu_hook)(void (*)(int, int, unsigned, unsigned));
static unsigned (*p_get_reg)(int);
#define REG_A7 15   /* m68k_register_t in the core's m68k.h */

/* coverage */
static uint8_t *cover;
static size_t rom_size;
static unsigned last_pc;
static unsigned ignore_pc[16];
static int n_ignore;
/* --watch */
static unsigned watch_addr[16];
static int n_watch;
static long watch_left = 64;
static long cur_frame;
/* --ram: the stack pointer's lowest value so far */
static int track_sp;
static unsigned min_sp = 0xFFFFFFFF;
/* --pcs */
static FILE *pcs_file;
static long pcs_from, pcs_to = 1;

enum {
    HOOK_M68K_E = 1 << 0,
    HOOK_M68K_R = 1 << 1,
    HOOK_M68K_W = 1 << 2,
    /* the core's cpuhook.h */
    HOOK_Z80_E = 1 << 9,
    HOOK_Z80_R = 1 << 10,
    HOOK_Z80_W = 1 << 11,
};

/* --snap: calls of one routine, entered and returned; --snaps: of many */
static unsigned snap_addr = 0xFFFFFFFF;
static const char *snap_out = "snap-";
static int snap_limit = 50, snap_n;
static const char *snaps_path;
#define SNAPS_MAX 8192
static unsigned snaps[SNAPS_MAX];
static int snaps_n, snaps_count[SNAPS_MAX];
static uint8_t *snaps_mark;     /* a bit for each address, 16 MB of them */
#define SNAP_DEPTH 64
static struct { unsigned sp, ret; int n, which; } snap_open[SNAP_DEPTH];
static int snap_open_n;

static void snap_write(int which, int n, const char *what)
{
    char path[1024];
    FILE *f;
    int r;
    const uint8_t *w = p_get_memory_data(RETRO_MEMORY_SYSTEM_RAM);
    size_t k, size = p_get_memory_size(RETRO_MEMORY_SYSTEM_RAM);
    if (which < 0)
        snprintf(path, sizeof path, "%s%d-%s.bin", snap_out, n, what);
    else
        snprintf(path, sizeof path, "%s%06X-%d-%s.bin", snap_out,
                 snaps[which], n, what);
    f = fopen(path, "wb");
    if (!f)
        return;
    for (r = 0; r < 18; r++) {
        unsigned v = p_get_reg(r);
        uint8_t b[4] = { v >> 24, v >> 16, v >> 8, v };
        fwrite(b, 1, 4, f);
    }
    for (k = 0; k + 1 < size; k += 2) {
        uint16_t v;
        uint8_t b[2];
        memcpy(&v, w + k, 2);
        b[0] = v >> 8;
        b[1] = v & 0xFF;
        fwrite(b, 1, 2, f);
    }
    fclose(f);
}

static void snaps_load(void)
{
    FILE *f = fopen(snaps_path, "r");
    char line[64];
    if (!f) {
        perror(snaps_path);
        exit(1);
    }
    snaps_mark = calloc(1 << 21, 1);
    while (snaps_n < SNAPS_MAX && fgets(line, sizeof line, f)) {
        char *end;
        unsigned a = strtoul(line, &end, 16) & 0xFFFFFF;
        if (end == line)
            continue;
        snaps[snaps_n++] = a;
        snaps_mark[a >> 3] |= 1 << (a & 7);
    }
    fclose(f);
}

static void snap_step(unsigned pc)
{
    unsigned sp = p_get_reg(REG_A7) & 0xFFFFFF;
    int which = -1, n;
    while (snap_open_n) {
        unsigned top_sp = snap_open[snap_open_n - 1].sp;
        if (pc == snap_open[snap_open_n - 1].ret && sp == top_sp + 4) {
            snap_open_n--;
            snap_write(snap_open[snap_open_n].which,
                       snap_open[snap_open_n].n, "out");
        } else if (sp > top_sp + 4) {
            /* its frame went without its return (a yield, a return
               from the caller, sp reloaded): the call never ends, and
               the ones below it may */
            snap_open_n--;
        } else
            break;
    }
    if (pc == snap_addr) {
        if (snap_n >= snap_limit)
            return;
        n = snap_n++;
    } else if (snaps_mark && (snaps_mark[pc >> 3] >> (pc & 7) & 1)) {
        int k;
        for (k = 0; k < snaps_n && snaps[k] != pc; k++)
            ;
        if (k == snaps_n || snaps_count[k] >= snap_limit)
            return;
        which = k;
        n = snaps_count[k]++;
    } else
        return;
    if (snap_open_n < SNAP_DEPTH) {
        const uint8_t *w = p_get_memory_data(RETRO_MEMORY_SYSTEM_RAM);
        unsigned a = sp & 0xFFFF, ret = 0;
        int k;
        for (k = 0; k < 4; k++) {
            unsigned b = (a + k) & 0xFFFF;
            ret = ret << 8 | w[b ^ 1];
        }
        snap_open[snap_open_n].sp = sp;
        snap_open[snap_open_n].ret = ret & 0xFFFFFF;
        snap_open[snap_open_n].n = n;
        snap_open[snap_open_n].which = which;
        snap_open_n++;
        snap_write(which, n, "in");
    }
}

/* --edges: a set of (from, to) pairs, open addressing */
#define EDGE_BITS 20
static unsigned long long *edges;
static unsigned long n_edges;
static unsigned prev_pc = 0xFFFFFFFF;

static void edge(unsigned from, unsigned to)
{
    unsigned long long key = (unsigned long long)from << 32 | to | 1ULL << 63;
    unsigned long h = (unsigned long)((key * 0x9E3779B97F4A7C15ULL) >> (64 - EDGE_BITS));
    while (edges[h]) {
        if (edges[h] == key)
            return;
        h = (h + 1) & ((1UL << EDGE_BITS) - 1);
    }
    if (n_edges < (1UL << EDGE_BITS) / 2) {
        edges[h] = key;
        n_edges++;
    }
}

/* --cycles: per ROM word, the costs seen and how often */
#define CSLOTS 8
struct cslot { uint32_t cost; uint32_t count; };
static struct cslot (*cyc)[CSLOTS];
static unsigned long cyc_overflow;
static unsigned cyc_pc = 0xFFFFFFFF, cyc_clock;
static int cyc_refresh;
static unsigned vectors[64];

/* --timeline */
static FILE *tl_file;
static long tl_at = -1;
static unsigned long tl_count;
static uint64_t tl_hash = 0xCBF29CE484222325ULL;
#define FRAME_CLOCKS 896040UL

static void timeline_step(unsigned pc)
{
    uint64_t clock = (uint64_t)cur_frame * FRAME_CLOCKS + p_m68k_cycles();
    uint64_t v = (uint64_t)pc << 40 ^ clock;
    int k;
    for (k = 0; k < 8; k++) {
        tl_hash ^= (v >> (8 * k)) & 0xFF;
        tl_hash *= 1099511628211ULL;
    }
    tl_count++;
    if (cur_frame == tl_at)
        fprintf(tl_file, "%06X %llu\n", pc, (unsigned long long)clock);
}

/* --ztimeline */
static FILE *zl_file;
static unsigned long zl_count;
static uint64_t zl_hash = 0xCBF29CE484222325ULL;

static void ztimeline_step(unsigned pc)
{
    uint64_t clock = (uint64_t)cur_frame * FRAME_CLOCKS + p_z80_cycles();
    uint64_t v = (uint64_t)pc << 40 ^ clock;
    int k;
    for (k = 0; k < 8; k++) {
        zl_hash ^= (v >> (8 * k)) & 0xFF;
        zl_hash *= 1099511628211ULL;
    }
    zl_count++;
    if (cur_frame == tl_at)
        fprintf(zl_file, "%04X %llu %d\n", pc, (unsigned long long)clock,
                p_ym_timer_b());
}

static void ztimeline_frame(long frame)
{
    fprintf(zl_file, "FRAME %ld %lu %016llX\n", frame, zl_count,
            (unsigned long long)zl_hash);
    zl_count = 0;
    zl_hash = 0xCBF29CE484222325ULL;
}

static void timeline_frame(long frame)
{
    fprintf(tl_file, "FRAME %ld %lu %016llX\n", frame, tl_count,
            (unsigned long long)tl_hash);
    tl_count = 0;
    tl_hash = 0xCBF29CE484222325ULL;
}

static void cycles_step(unsigned pc)
{
    unsigned c = p_m68k_cycles();
    int rf = p_m68k_refresh();
    int k, exception = 0;
    for (k = 2; k < 64; k++)
        if (vectors[k] == pc)
            exception = 1;
    if (cyc_pc < rom_size && c >= cyc_clock && !exception) {
        /* a refresh at the fetch: due by then, and it moves the next one
         * to 128 cycles from it */
        int refresh = cyc_clock >= (unsigned)cyc_refresh;
        unsigned d = c - cyc_clock - (refresh ? 14 : 0);
        int skip = refresh ? rf != (int)cyc_clock + 128 * 7
                           : rf != cyc_refresh;
        uint32_t cost = (d << 1) | skip;
        struct cslot *sl = cyc[cyc_pc >> 1];
        for (k = 0; k < CSLOTS; k++) {
            if (sl[k].count && sl[k].cost == cost) {
                sl[k].count++;
                break;
            }
            if (!sl[k].count) {
                sl[k].cost = cost;
                sl[k].count = 1;
                break;
            }
        }
        if (k == CSLOTS)
            cyc_overflow++;
    }
    cyc_pc = pc;
    cyc_clock = c;
    cyc_refresh = rf;
}

/* --zcover */
static uint8_t zcover[0x2000];

static void hook(int type, int width, unsigned address, unsigned value)
{
    if (type & (HOOK_Z80_E | HOOK_Z80_R | HOOK_Z80_W)) {
        if (type == HOOK_Z80_E)
            zcover[address & 0x1FFF] = 1;
        if (type == HOOK_Z80_E && zl_file)
            ztimeline_step(address & 0xFFFF);
        return;
    }
    address &= 0xFFFFFF;
    if (type == HOOK_M68K_E) {
        if (cyc)
            cycles_step(address);
        if (tl_file)
            timeline_step(address);
        if (snap_addr != 0xFFFFFFFF || snaps_mark)
            snap_step(address);
        if (edges && prev_pc != 0xFFFFFFFF &&
            (address <= prev_pc || address > prev_pc + 10))
            edge(prev_pc, address);
        prev_pc = address;
        last_pc = address;
        if (track_sp) {
            unsigned sp = p_get_reg(REG_A7) & 0xFFFFFF;
            if (sp >= 0xFF0000 && sp < min_sp)
                min_sp = sp;
        }
        if (pcs_file && cur_frame >= pcs_from && cur_frame < pcs_to)
            fprintf(pcs_file, "%ld %06X\n", cur_frame, address);
    } else if (n_watch && watch_left > 0) {
        int k;
        for (k = 0; k < n_watch; k++)
            if (watch_addr[k] >= address && watch_addr[k] < address + width) {
                fprintf(stderr, "frame %ld pc %06X %c%d %06X = %0*X\n",
                        cur_frame, last_pc,
                        type == HOOK_M68K_W ? 'W' : 'R', width, address,
                        width * 2, value);
                watch_left--;
                break;
            }
    }
    if (address >= rom_size)
        return;
    if (type == HOOK_M68K_R && pcs_file && cur_frame >= pcs_from &&
        cur_frame < pcs_to)
        fprintf(pcs_file, "%ld %06X R%d %0*X\n", cur_frame, address, width,
                width * 2, value);
    if (type == HOOK_M68K_E) {
        last_pc = address;
        cover[address] |= 0x01;
    } else if (type == HOOK_M68K_R) {
        unsigned n = width, i;
        int k;
        for (k = 0; k < n_ignore; k++)
            if (ignore_pc[k] == last_pc)
                return;
        cover[address] |= width == 1 ? 0x02 : width == 2 ? 0x04 : 0x08;
        for (i = 0; i < n && address + i < rom_size; i++)
            cover[address + i] |= 0x10;
    } else if (type == HOOK_M68K_W) {
        cover[address] |= 0x20;
    }
}

/* video */
static enum retro_pixel_format pixfmt = RETRO_PIXEL_FORMAT_0RGB1555;
static uint8_t *frame_rgb;
static unsigned frame_w, frame_h;

static void video(const void *data, unsigned w, unsigned h, size_t pitch)
{
    unsigned x, y;
    if (!data)
        return;
    frame_w = w;
    frame_h = h;
    frame_rgb = realloc(frame_rgb, (size_t)w * h * 3);
    for (y = 0; y < h; y++) {
        const uint8_t *row = (const uint8_t *)data + y * pitch;
        uint8_t *o = frame_rgb + (size_t)y * w * 3;
        for (x = 0; x < w; x++, o += 3) {
            if (pixfmt == RETRO_PIXEL_FORMAT_XRGB8888) {
                uint32_t p = ((const uint32_t *)row)[x];
                o[0] = p >> 16;
                o[1] = p >> 8;
                o[2] = p;
            } else if (pixfmt == RETRO_PIXEL_FORMAT_RGB565) {
                uint16_t p = ((const uint16_t *)row)[x];
                o[0] = ((p >> 11) & 31) * 255 / 31;
                o[1] = ((p >> 5) & 63) * 255 / 63;
                o[2] = (p & 31) * 255 / 31;
            } else {
                uint16_t p = ((const uint16_t *)row)[x];
                o[0] = ((p >> 10) & 31) * 255 / 31;
                o[1] = ((p >> 5) & 31) * 255 / 31;
                o[2] = (p & 31) * 255 / 31;
            }
        }
    }
}

static void put32(uint8_t *p, uint32_t v)
{
    p[0] = v >> 24;
    p[1] = v >> 16;
    p[2] = v >> 8;
    p[3] = v;
}

static void png_chunk(FILE *f, const char *type, const uint8_t *d, uint32_t n)
{
    uint8_t b[4];
    uint32_t crc;
    put32(b, n);
    fwrite(b, 1, 4, f);
    fwrite(type, 1, 4, f);
    if (n)
        fwrite(d, 1, n, f);
    crc = crc32(0, (const uint8_t *)type, 4);
    if (n)
        crc = crc32(crc, d, n);
    put32(b, crc);
    fwrite(b, 1, 4, f);
}

static void write_png(const char *path)
{
    FILE *f;
    uint8_t ihdr[13];
    size_t raw_n = (size_t)(frame_w * 3 + 1) * frame_h, y;
    uLongf z_n = compressBound(raw_n);
    uint8_t *raw = malloc(raw_n), *z = malloc(z_n);
    if (!frame_rgb)
        return;
    for (y = 0; y < frame_h; y++) {
        raw[y * (frame_w * 3 + 1)] = 0;
        memcpy(raw + y * (frame_w * 3 + 1) + 1, frame_rgb + y * frame_w * 3,
               frame_w * 3);
    }
    compress2(z, &z_n, raw, raw_n, 6);
    f = fopen(path, "wb");
    if (!f) {
        perror(path);
        exit(1);
    }
    fwrite("\x89PNG\r\n\x1a\n", 1, 8, f);
    put32(ihdr, frame_w);
    put32(ihdr + 4, frame_h);
    ihdr[8] = 8;
    ihdr[9] = 2;
    ihdr[10] = ihdr[11] = ihdr[12] = 0;
    png_chunk(f, "IHDR", ihdr, 13);
    png_chunk(f, "IDAT", z, z_n);
    png_chunk(f, "IEND", NULL, 0);
    fclose(f);
    free(raw);
    free(z);
}

/* input */
struct event {
    long frame;
    int pad;
    uint16_t mask;
};
static struct event events[MAX_EVENTS];
static int n_events, next_event;
/* RAM writes */
struct poke {
    long frame;
    unsigned addr;
    int size;
    uint32_t value;
};
static struct poke pokes[MAX_EVENTS];
static int n_pokes, next_poke;
static uint16_t pads[8];

static uint16_t parse_buttons(const char *s, const char *path, int line)
{
    /* libretro joypad ids the core maps to the MD's buttons */
    uint16_t m = 0;
    for (; *s; s++) {
        switch (*s) {
        case 'U': m |= 1 << RETRO_DEVICE_ID_JOYPAD_UP; break;
        case 'D': m |= 1 << RETRO_DEVICE_ID_JOYPAD_DOWN; break;
        case 'L': m |= 1 << RETRO_DEVICE_ID_JOYPAD_LEFT; break;
        case 'R': m |= 1 << RETRO_DEVICE_ID_JOYPAD_RIGHT; break;
        case 'A': m |= 1 << RETRO_DEVICE_ID_JOYPAD_Y; break;
        case 'B': m |= 1 << RETRO_DEVICE_ID_JOYPAD_B; break;
        case 'C': m |= 1 << RETRO_DEVICE_ID_JOYPAD_A; break;
        case 'S': m |= 1 << RETRO_DEVICE_ID_JOYPAD_START; break;
        case 'X': m |= 1 << RETRO_DEVICE_ID_JOYPAD_L; break;
        case 'Y': m |= 1 << RETRO_DEVICE_ID_JOYPAD_X; break;
        case 'Z': m |= 1 << RETRO_DEVICE_ID_JOYPAD_R; break;
        case 'M': m |= 1 << RETRO_DEVICE_ID_JOYPAD_SELECT; break;
        case '-': break;
        default:
            fprintf(stderr, "%s:%d: unknown button '%c'\n", path, line, *s);
            exit(1);
        }
    }
    return m;
}

static void load_input(const char *path)
{
    FILE *f = fopen(path, "r");
    char buf[256];
    int line = 0;
    if (!f) {
        perror(path);
        exit(1);
    }
    while (fgets(buf, sizeof buf, f)) {
        char *h = strchr(buf, '#'), *tok;
        long frame;
        int pad = 1;
        line++;
        if (h)
            *h = 0;
        tok = strtok(buf, " \t\r\n");
        if (!tok)
            continue;
        frame = strtol(tok, NULL, 10);
        tok = strtok(NULL, " \t\r\n");
        if (!tok) {
            fprintf(stderr, "%s:%d: no buttons\n", path, line);
            exit(1);
        }
        if (tok[0] == '=') {
            char *val = strtok(NULL, " \t\r\n"), *end;
            struct poke *k = &pokes[n_pokes];
            size_t len = val ? strlen(val) : 0;
            k->frame = frame;
            k->addr = strtoul(tok + 1, &end, 16) & 0xFFFFFF;
            if (*end || k->addr < 0xFF0000) {
                fprintf(stderr, "%s:%d: bad RAM address\n", path, line);
                exit(1);
            }
            k->value = val ? strtoul(val, &end, 16) : 0;
            k->size = len / 2;
            if (!val || *end || (len != 2 && len != 4 && len != 8) ||
                    (k->size > 1 && k->addr & 1)) {
                fprintf(stderr, "%s:%d: want =ADDR VALUE (2, 4 or 8 hex "
                        "digits; even ADDR for a word or long)\n", path, line);
                exit(1);
            }
            if (n_pokes && frame < pokes[n_pokes - 1].frame) {
                fprintf(stderr, "%s:%d: frames go backwards\n", path, line);
                exit(1);
            }
            if (++n_pokes == MAX_EVENTS) {
                fprintf(stderr, "%s: too many lines\n", path);
                exit(1);
            }
            continue;
        }
        if (tok[0] >= '1' && tok[0] <= '8' && tok[1] == ':') {
            pad = tok[0] - '0';
            tok += 2;
        }
        if (n_events && frame < events[n_events - 1].frame) {
            fprintf(stderr, "%s:%d: frames go backwards\n", path, line);
            exit(1);
        }
        if (n_events == MAX_EVENTS) {
            fprintf(stderr, "%s: too many lines\n", path);
            exit(1);
        }
        events[n_events].frame = frame;
        events[n_events].pad = pad - 1;
        events[n_events].mask = parse_buttons(tok, path, line);
        n_events++;
    }
    fclose(f);
}

static void poke(const struct poke *k)
{
    /* the core keeps work RAM as host-order 16-bit words */
    uint8_t *w = p_get_memory_data(RETRO_MEMORY_SYSTEM_RAM);
    int i;
    for (i = 0; i < k->size; i++) {
        unsigned a = (k->addr + i) & 0xFFFF;
        uint16_t v;
        int shift = 8 * (k->size - 1 - i);
        memcpy(&v, w + (a & ~1u), 2);
        if (a & 1)
            v = (v & 0xFF00) | ((k->value >> shift) & 0xFF);
        else
            v = (v & 0x00FF) | (((k->value >> shift) & 0xFF) << 8);
        memcpy(w + (a & ~1u), &v, 2);
    }
}

static void poll(void) {}

static int16_t state(unsigned port, unsigned device, unsigned index,
                     unsigned id)
{
    (void)index;
    if ((device & RETRO_DEVICE_MASK) != RETRO_DEVICE_JOYPAD || port >= 8)
        return 0;
    if (id == RETRO_DEVICE_ID_JOYPAD_MASK)
        return pads[port];
    return (pads[port] >> id) & 1;
}

/* --wav: 16-bit stereo, the header written at the end */
static FILE *wav_file;
static uint32_t wav_frames;

static void audio_batch_write(const int16_t *d, size_t n)
{
    if (wav_file) {
        fwrite(d, 4, n, wav_file);
        wav_frames += n;
    }
}

static void audio(int16_t l, int16_t r)
{
    int16_t d[2] = {l, r};
    audio_batch_write(d, 1);
}

static size_t audio_batch(const int16_t *d, size_t n)
{
    audio_batch_write(d, n);
    return n;
}

static void wav32(uint8_t *p, uint32_t v)
{
    p[0] = v; p[1] = v >> 8; p[2] = v >> 16; p[3] = v >> 24;
}

static void wav_header(FILE *f, uint32_t rate, uint32_t n)
{
    uint8_t h[45] = "RIFF....WAVEfmt ....\1\0\2\0........\4\0\20\0data....";
    wav32(h + 4, 36 + n * 4);
    wav32(h + 16, 16);
    wav32(h + 24, rate);
    wav32(h + 28, rate * 4);
    wav32(h + 40, n * 4);
    fseek(f, 0, SEEK_SET);
    fwrite(h, 1, 44, f);
}

static void core_log(enum retro_log_level level, const char *fmt, ...)
{
    va_list ap;
    if (level < RETRO_LOG_WARN)
        return;
    va_start(ap, fmt);
    vfprintf(stderr, fmt, ap);
    va_end(ap);
}

static const char *dir = ".";

static bool env(unsigned cmd, void *data)
{
    switch (cmd) {
    case RETRO_ENVIRONMENT_GET_LOG_INTERFACE:
        ((struct retro_log_callback *)data)->log = core_log;
        return true;
    case RETRO_ENVIRONMENT_SET_PIXEL_FORMAT:
        pixfmt = *(enum retro_pixel_format *)data;
        return true;
    case RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY:
    case RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY:
        *(const char **)data = dir;
        return true;
    case RETRO_ENVIRONMENT_GET_INPUT_BITMASKS:
        return true;
    case RETRO_ENVIRONMENT_GET_VARIABLE: {
        struct retro_variable *v = data;
        v->value = NULL;
        if (!strcmp(v->key, "genesis_plus_gx_region_detect")) {
            v->value = region;
            return true;
        }
        /* save files would land in dir: none */
        if (!strcmp(v->key, "genesis_plus_gx_bram"))
            return false;
        return false;
    }
    case RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE:
        *(bool *)data = false;
        return true;
    case RETRO_ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE:
        *(int *)data = 3;
        return true;
    default:
        return false;
    }
}

static void *sym(void *h, const char *name)
{
    void *p = dlsym(h, name);
    if (!p) {
        fprintf(stderr, "core lacks %s (built by tools/get_gpgx.sh?)\n", name);
        exit(1);
    }
    return p;
}

static uint8_t *read_file(const char *path, size_t *n)
{
    FILE *f = fopen(path, "rb");
    uint8_t *d;
    long len;
    if (!f) {
        perror(path);
        exit(1);
    }
    fseek(f, 0, SEEK_END);
    len = ftell(f);
    fseek(f, 0, SEEK_SET);
    d = malloc(len ? len : 1);
    if (fread(d, 1, len, f) != (size_t)len) {
        perror(path);
        exit(1);
    }
    fclose(f);
    *n = len;
    return d;
}

/* --hashes */
static FILE *hash_file;

static void hashes(long frame)
{
    const uint8_t *w = p_get_memory_data(RETRO_MEMORY_SYSTEM_RAM);
    uint8_t be[0xFE00];
    size_t k;
    for (k = 0; k < sizeof be; k += 2) {
        uint16_t v;
        memcpy(&v, w + k, 2);
        be[k] = v >> 8;
        be[k + 1] = v & 0xff;
    }
    fprintf(hash_file, "%ld %08lX %08lX\n", frame,
            crc32(0, be, sizeof be),
            frame_rgb ? crc32(0, frame_rgb, frame_w * frame_h * 3) : 0UL);
}

static unsigned char *(*p_vdp)(int);
static const char *vram_prefix;

static void dump_be(const char *path, const uint8_t *host, size_t n)
{
    FILE *f = fopen(path, "wb");
    size_t k;
    if (!f) {
        perror(path);
        exit(1);
    }
    for (k = 0; k + 1 < n; k += 2) {
        uint16_t v;
        memcpy(&v, host + k, 2);
        fputc(v >> 8, f);
        fputc(v & 0xff, f);
    }
    fclose(f);
}

static void snapshot(const char *shot, const char *ram, long frame)
{
    char path[1024];
    if (vram_prefix) {
        FILE *f;
        snprintf(path, sizeof path, "%s%06ld.vram", vram_prefix, frame);
        dump_be(path, p_vdp(0), 0x10000);
        snprintf(path, sizeof path, "%s%06ld.vsram", vram_prefix, frame);
        dump_be(path, p_vdp(1), 0x80);
        snprintf(path, sizeof path, "%s%06ld.sat", vram_prefix, frame);
        dump_be(path, p_vdp(3), 0x400);
        snprintf(path, sizeof path, "%s%06ld.regs", vram_prefix, frame);
        f = fopen(path, "wb");
        if (f) {
            fwrite(p_vdp(2), 1, 0x20, f);
            fclose(f);
        }
    }
    if (shot) {
        snprintf(path, sizeof path, "%s%06ld.png", shot, frame);
        write_png(path);
    }
    if (ram) {
        FILE *f;
        snprintf(path, sizeof path, "%s%06ld.bin", ram, frame);
        f = fopen(path, "wb");
        if (!f) {
            perror(path);
            exit(1);
        }
        {
            /* the core keeps work RAM as host-order 16-bit words: write it
               as the 68000 sees it, big-endian */
            const uint8_t *w = p_get_memory_data(RETRO_MEMORY_SYSTEM_RAM);
            size_t k, n = p_get_memory_size(RETRO_MEMORY_SYSTEM_RAM);
            uint8_t *be = malloc(n);
            for (k = 0; k + 1 < n; k += 2) {
                uint16_t v;
                memcpy(&v, w + k, 2);
                be[k] = v >> 8;
                be[k + 1] = v & 0xff;
            }
            fwrite(be, 1, n, f);
            free(be);
        }
        fclose(f);
        /* the stack pointer now and the lowest it has been: RAM between
           the two is dead stack */
        snprintf(path, sizeof path, "%s%06ld.sp", ram, frame);
        f = fopen(path, "w");
        if (!f) {
            perror(path);
            exit(1);
        }
        fprintf(f, "%06X %06X\n", p_get_reg(REG_A7) & 0xFFFFFF,
                min_sp & 0xFFFFFF);
        fclose(f);
    }
}

int main(int argc, char **argv)
{
    const char *core = "tools/gpgx/genesis_plus_gx_libretro.so";
    const char *rom_path = NULL, *input = NULL, *cover_path = NULL;
    const char *shot = NULL, *ram = NULL, *log_path = NULL, *wav_path = NULL,
               *cycles_path = NULL, *timeline_path = NULL,
               *ztimeline_path = NULL, *hashes_path = NULL,
               *zcover_path = NULL;
    const char *edges_path = NULL;
    long frames = 3600, every = 600, frame;
    struct retro_game_info info;
    void *h;
    uint8_t *rom;
    FILE *logf = NULL;
    int i;

    for (i = 1; i < argc; i++) {
        const char *a = argv[i];
#define ARG(name) (!strcmp(a, name) && i + 1 < argc)
        if (ARG("--core")) core = argv[++i];
        else if (ARG("--frames")) frames = strtol(argv[++i], NULL, 0);
        else if (ARG("--input")) input = argv[++i];
        else if (ARG("--cover")) cover_path = argv[++i];
        else if (ARG("--shot")) shot = argv[++i];
        else if (ARG("--wav")) wav_path = argv[++i];
        else if (ARG("--cycles")) cycles_path = argv[++i];
        else if (ARG("--timeline")) timeline_path = argv[++i];
        else if (ARG("--timeline-at")) tl_at = strtol(argv[++i], NULL, 0);
        else if (ARG("--ztimeline")) ztimeline_path = argv[++i];
        else if (ARG("--zcover")) zcover_path = argv[++i];
        else if (ARG("--hashes")) hashes_path = argv[++i];
        else if (ARG("--vram")) vram_prefix = argv[++i];
        else if (ARG("--shot-every")) every = strtol(argv[++i], NULL, 0);
        else if (ARG("--ram")) ram = argv[++i];
        else if (ARG("--region")) {
            const char *r = argv[++i];
            region = !strcmp(r, "ntsc-u") ? "ntsc-u" : !strcmp(r, "pal") ? "pal"
                   : !strcmp(r, "ntsc-j") ? "ntsc-j" : "auto";
        }
        else if (ARG("--log")) log_path = argv[++i];
        else if (ARG("--edges")) edges_path = argv[++i];
        else if (ARG("--snap")) snap_addr = strtoul(argv[++i], NULL, 16);
        else if (ARG("--snaps")) snaps_path = argv[++i];
        else if (ARG("--snap-out")) snap_out = argv[++i];
        else if (ARG("--snap-limit")) snap_limit = strtol(argv[++i], NULL, 0);
        else if (ARG("--ignore-reads") && n_ignore < 16)
            ignore_pc[n_ignore++] = strtoul(argv[++i], NULL, 16);
        else if (ARG("--watch") && n_watch < 16)
            watch_addr[n_watch++] = strtoul(argv[++i], NULL, 16) & 0xFFFFFF;
        else if (ARG("--watch-limit")) watch_left = strtol(argv[++i], NULL, 0);
        else if (ARG("--pcs")) {
            pcs_file = fopen(argv[++i], "w");
            if (!pcs_file) {
                perror(argv[i]);
                return 1;
            }
        }
        else if (ARG("--pcs-frames")) {
            char *e;
            pcs_from = strtol(argv[++i], &e, 0);
            pcs_to = *e == ':' ? strtol(e + 1, NULL, 0) : pcs_from + 1;
        }
        else if (!strcmp(a, "--teamplayer")) teamplayer = 1;
        else if (!strcmp(a, "--wayplay")) wayplay = 1;
        else if (a[0] == '-') {
            fprintf(stderr, "unknown option %s (see the comment in %s)\n", a,
                    __FILE__);
            return 2;
        } else rom_path = a;
#undef ARG
    }
    if (!rom_path) {
        fprintf(stderr, "usage: mdtrace [options] ROM\n");
        return 2;
    }
    if (input)
        load_input(input);

    h = dlopen(core, RTLD_NOW | RTLD_LOCAL);
    if (!h) {
        fprintf(stderr, "%s\n", dlerror());
        return 1;
    }
    p_retro_init = sym(h, "retro_init");
    p_retro_deinit = sym(h, "retro_deinit");
    p_set_env = sym(h, "retro_set_environment");
    p_set_video = sym(h, "retro_set_video_refresh");
    p_set_audio = sym(h, "retro_set_audio_sample");
    p_set_audio_batch = sym(h, "retro_set_audio_sample_batch");
    p_set_poll = sym(h, "retro_set_input_poll");
    p_set_state = sym(h, "retro_set_input_state");
    p_load_game = sym(h, "retro_load_game");
    p_run = sym(h, "retro_run");
    p_get_av_info = sym(h, "retro_get_system_av_info");
    p_m68k_cycles = sym(h, "mdtrace_m68k_cycles");
    p_m68k_refresh = sym(h, "mdtrace_m68k_refresh");
    p_z80_cycles = sym(h, "mdtrace_z80_cycles");
    p_ym_timer_b = sym(h, "mdtrace_ym_timer_b");
    p_vdp = sym(h, "mdtrace_vdp");
    p_get_memory_data = sym(h, "retro_get_memory_data");
    p_get_memory_size = sym(h, "retro_get_memory_size");
    p_set_port = sym(h, "retro_set_controller_port_device");
    p_set_cpu_hook = sym(h, "set_cpu_hook");
    p_get_reg = sym(h, "m68k_get_reg");
    track_sp = ram != NULL;

    rom = read_file(rom_path, &rom_size);
    if (rom_size > ROM_MAX)
        rom_size = ROM_MAX;
    cover = calloc(rom_size, 1);
    if (cycles_path) {
        int k;
        cyc = calloc(rom_size / 2 + 1, sizeof *cyc);
        for (k = 2; k < 64 && 4 * k + 3 < (int)rom_size; k++)
            vectors[k] = (rom[4 * k] << 24 | rom[4 * k + 1] << 16 |
                          rom[4 * k + 2] << 8 | rom[4 * k + 3]) & 0xFFFFFF;
    }
    if (cover_path) {
        FILE *f = fopen(cover_path, "rb");
        if (f) {
            uint8_t *old = malloc(rom_size);
            if (fread(old, 1, rom_size, f) == rom_size) {
                size_t k;
                for (k = 0; k < rom_size; k++)
                    cover[k] |= old[k];
            }
            free(old);
            fclose(f);
        }
    }

    p_set_env(env);
    p_retro_init();
    p_set_video(video);
    p_set_audio(audio);
    p_set_audio_batch(audio_batch);
    p_set_poll(poll);
    p_set_state(state);
    memset(&info, 0, sizeof info);
    info.path = rom_path;
    info.data = rom;
    info.size = rom_size;
    if (!p_load_game(&info)) {
        fprintf(stderr, "the core refused %s\n", rom_path);
        return 1;
    }
    /* libretro's 3-button pad, or the Team Player (subclass 5) on port 1 */
    /* the core's MD Joypad 3 Button (0), + 4-WayPlay (3, both ports), +
     * Teamplayer (5) */
    {
        /* (the macro doesn't parenthesize its id: computed apart) */
        unsigned sub = wayplay ? 3 : teamplayer ? 5 : 0;
        p_set_port(0, RETRO_DEVICE_SUBCLASS(RETRO_DEVICE_JOYPAD, sub));
    }
    if (wayplay)    /* the core takes it on either port; both, to be sure */
        p_set_port(1, RETRO_DEVICE_SUBCLASS(RETRO_DEVICE_JOYPAD, 3));
    else if (!teamplayer)
        p_set_port(1, RETRO_DEVICE_SUBCLASS(RETRO_DEVICE_JOYPAD, 0));
    if (snaps_path)
        snaps_load();
    p_set_cpu_hook(hook);
    if (edges_path) {
        edges = calloc(1UL << EDGE_BITS, sizeof *edges);
        if (!edges) {
            perror("calloc");
            return 1;
        }
    }
    if (log_path) {
        logf = fopen(log_path, "w");
        if (!logf) {
            perror(log_path);
            return 1;
        }
    }

    if (timeline_path) {
        tl_file = fopen(timeline_path, "w");
        if (!tl_file) {
            perror(timeline_path);
            return 1;
        }
    }
    if (hashes_path) {
        hash_file = fopen(hashes_path, "w");
        if (!hash_file) {
            perror(hashes_path);
            return 1;
        }
    }
    if (ztimeline_path) {
        zl_file = fopen(ztimeline_path, "w");
        if (!zl_file) {
            perror(ztimeline_path);
            return 1;
        }
    }
    if (wav_path) {
        wav_file = fopen(wav_path, "wb");
        if (!wav_file) {
            perror(wav_path);
            return 1;
        }
        wav_header(wav_file, 0, 0);
    }

    for (frame = 0; frame < frames; frame++) {
        cur_frame = frame;
        while (next_event < n_events && events[next_event].frame <= frame) {
            pads[events[next_event].pad] = events[next_event].mask;
            next_event++;
        }
        while (next_poke < n_pokes && pokes[next_poke].frame <= frame)
            poke(&pokes[next_poke++]);
        p_run();
        if (tl_file)
            timeline_frame(frame);
        if (zl_file)
            ztimeline_frame(frame);
        if (hash_file)
            hashes(frame);
        if (logf) {
            uint8_t *wram = p_get_memory_data(RETRO_MEMORY_SYSTEM_RAM);
            fprintf(logf, "%ld %04x %04x %04x %04x %08lx\n", frame, pads[0],
                    pads[1], pads[2], pads[3],
                    crc32(0, wram, p_get_memory_size(RETRO_MEMORY_SYSTEM_RAM)));
        }
        if ((shot || ram || vram_prefix) && every > 0 && (frame + 1) % every == 0)
            snapshot(shot, ram, frame + 1);
    }
    if ((shot || ram || vram_prefix) && (every <= 0 || frames % every))
        snapshot(shot, ram, frames);

    if (tl_file)
        fclose(tl_file);
    if (zl_file)
        fclose(zl_file);
    if (zcover_path) {
        FILE *f = fopen(zcover_path, "wb");
        if (!f || fwrite(zcover, 1, sizeof zcover, f) != sizeof zcover) {
            perror(zcover_path);
            return 1;
        }
        fclose(f);
    }
    if (hash_file)
        fclose(hash_file);
    if (cycles_path) {
        FILE *f = fopen(cycles_path, "w");
        size_t a;
        int k;
        if (!f) {
            perror(cycles_path);
            return 1;
        }
        for (a = 0; a < rom_size / 2; a++)
            for (k = 0; k < CSLOTS && cyc[a][k].count; k++)
                fprintf(f, "%06zX %u %u %u\n", 2 * a, cyc[a][k].cost >> 1,
                        cyc[a][k].cost & 1, cyc[a][k].count);
        if (cyc_overflow)
            fprintf(stderr, "--cycles: %lu instructions with more than %d "
                    "costs left out\n", cyc_overflow, CSLOTS);
        fclose(f);
    }

    if (wav_file) {
        struct retro_system_av_info av;
        p_get_av_info(&av);
        wav_header(wav_file, (uint32_t)(av.timing.sample_rate + 0.5), wav_frames);
        fclose(wav_file);
    }

    if (cover_path) {
        FILE *f = fopen(cover_path, "wb");
        if (!f) {
            perror(cover_path);
            return 1;
        }
        fwrite(cover, 1, rom_size, f);
        fclose(f);
    }
    if (logf)
        fclose(logf);
    if (pcs_file)
        fclose(pcs_file);
    if (edges_path) {
        FILE *f = fopen(edges_path, "w");
        unsigned long h;
        if (!f) {
            perror(edges_path);
            return 1;
        }
        for (h = 0; h < 1UL << EDGE_BITS; h++)
            if (edges[h])
                fprintf(f, "%06X %06X\n",
                        (unsigned)(edges[h] >> 32) & 0xFFFFFF,
                        (unsigned)edges[h] & 0xFFFFFF);
        fclose(f);
    }
    p_set_cpu_hook(NULL);
    p_retro_deinit();
    return 0;
}
