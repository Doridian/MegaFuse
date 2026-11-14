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
 *     --ram PREFIX       work RAM (64 KB) as PREFIX<frame>.bin, same frames
 *     --region R         auto (default), ntsc-u, pal, ntsc-j
 *     --teamplayer       Sega's Team Player on port 1 (pads 1-4 on it)
 *     --log FILE         every frame's pad state and the RAM hash
 *     --ignore-reads PC  don't record the reads the instruction at PC makes
 *                        (repeatable; the boot checksum reads the whole ROM)
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
static void *(*p_get_memory_data)(unsigned);
static size_t (*p_get_memory_size)(unsigned);
static void (*p_set_port)(unsigned, unsigned);
static void (*p_set_cpu_hook)(void (*)(int, int, unsigned, unsigned));

/* coverage */
static uint8_t *cover;
static size_t rom_size;
static unsigned last_pc;
static unsigned ignore_pc[16];
static int n_ignore;

enum {
    HOOK_M68K_E = 1 << 0,
    HOOK_M68K_R = 1 << 1,
    HOOK_M68K_W = 1 << 2,
};

static void hook(int type, int width, unsigned address, unsigned value)
{
    (void)value;
    address &= 0xFFFFFF;
    if (type == HOOK_M68K_E)
        last_pc = address;
    if (address >= rom_size)
        return;
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

static void audio(int16_t l, int16_t r) { (void)l; (void)r; }
static size_t audio_batch(const int16_t *d, size_t n) { (void)d; return n; }

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

static void snapshot(const char *shot, const char *ram, long frame)
{
    char path[1024];
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
        fwrite(p_get_memory_data(RETRO_MEMORY_SYSTEM_RAM), 1,
               p_get_memory_size(RETRO_MEMORY_SYSTEM_RAM), f);
        fclose(f);
    }
}

int main(int argc, char **argv)
{
    const char *core = "tools/gpgx/genesis_plus_gx_libretro.so";
    const char *rom_path = NULL, *input = NULL, *cover_path = NULL;
    const char *shot = NULL, *ram = NULL, *log_path = NULL;
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
        else if (ARG("--shot-every")) every = strtol(argv[++i], NULL, 0);
        else if (ARG("--ram")) ram = argv[++i];
        else if (ARG("--region")) {
            const char *r = argv[++i];
            region = !strcmp(r, "ntsc-u") ? "ntsc-u" : !strcmp(r, "pal") ? "pal"
                   : !strcmp(r, "ntsc-j") ? "ntsc-j" : "auto";
        }
        else if (ARG("--log")) log_path = argv[++i];
        else if (ARG("--ignore-reads") && n_ignore < 16)
            ignore_pc[n_ignore++] = strtoul(argv[++i], NULL, 16);
        else if (!strcmp(a, "--teamplayer")) teamplayer = 1;
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
    p_get_memory_data = sym(h, "retro_get_memory_data");
    p_get_memory_size = sym(h, "retro_get_memory_size");
    p_set_port = sym(h, "retro_set_controller_port_device");
    p_set_cpu_hook = sym(h, "set_cpu_hook");

    rom = read_file(rom_path, &rom_size);
    if (rom_size > ROM_MAX)
        rom_size = ROM_MAX;
    cover = calloc(rom_size, 1);
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
    p_set_port(0, teamplayer ? RETRO_DEVICE_SUBCLASS(RETRO_DEVICE_JOYPAD, 5)
                             : RETRO_DEVICE_SUBCLASS(RETRO_DEVICE_JOYPAD, 0));
    if (!teamplayer)
        p_set_port(1, RETRO_DEVICE_SUBCLASS(RETRO_DEVICE_JOYPAD, 0));
    p_set_cpu_hook(hook);
    if (log_path) {
        logf = fopen(log_path, "w");
        if (!logf) {
            perror(log_path);
            return 1;
        }
    }

    for (frame = 0; frame < frames; frame++) {
        while (next_event < n_events && events[next_event].frame <= frame) {
            pads[events[next_event].pad] = events[next_event].mask;
            next_event++;
        }
        p_run();
        if (logf) {
            uint8_t *wram = p_get_memory_data(RETRO_MEMORY_SYSTEM_RAM);
            fprintf(logf, "%ld %04x %04x %04x %04x %08lx\n", frame, pads[0],
                    pads[1], pads[2], pads[3],
                    crc32(0, wram, p_get_memory_size(RETRO_MEMORY_SYSTEM_RAM)));
        }
        if ((shot || ram) && every > 0 && (frame + 1) % every == 0)
            snapshot(shot, ram, frame + 1);
    }
    if ((shot || ram) && (every <= 0 || frames % every))
        snapshot(shot, ram, frames);

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
    p_set_cpu_hook(NULL);
    p_retro_deinit();
    return 0;
}
