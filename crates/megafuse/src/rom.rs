//! The cartridge the player gives the port: Mega Bomberman, the US
//! version (the one the translation is of: docs/PORT.md). The port
//! carries none of the game, so it asks for this at run time.

/// The cartridge's size.
const SIZE: usize = 0x10_0000;
/// The header's checksum word (0x18E), which tells the versions apart:
/// their headers are otherwise the same.
const CHECKSUM_US: u16 = 0x29F3;
const CHECKSUM_EU: u16 = 0xE962;
/// The domestic and overseas titles in the header.
const TITLE: &[u8] = b"MEGA BOMBER MAN";

/// Whether `rom` is a cartridge the port plays; why not if it isn't.
pub fn check(rom: &[u8]) -> Result<(), String> {
    if rom.len() != SIZE || !rom[0x120..].starts_with(TITLE) || !rom[0x150..].starts_with(TITLE) {
        return Err("not Mega Bomberman's ROM (a 1 MiB .bin, as dumped)".into());
    }
    match u16::from_be_bytes([rom[0x18E], rom[0x18F]]) {
        CHECKSUM_US => Ok(()),
        CHECKSUM_EU => Err("the European version; the port plays the US one".into()),
        _ => Err("an altered Mega Bomberman ROM; the port plays the US one, unaltered".into()),
    }
}

/// The ROM from the file dialog, asked again until it is one the port
/// plays; `None` if the player cancels (or there is no dialog to ask
/// with: Linux without a desktop portal or zenity).
#[cfg(not(target_arch = "wasm32"))]
pub fn pick() -> Option<(String, Vec<u8>)> {
    loop {
        let path = rfd::FileDialog::new()
            .set_title("Choose the Mega Bomberman ROM (US)")
            .add_filter("Mega Drive ROM", &["bin", "BIN", "md", "MD", "gen", "GEN"])
            .add_filter("All files", &["*"])
            .pick_file()?;
        let name = path.display().to_string();
        let why = match std::fs::read(&path) {
            Ok(rom) => match check(&rom) {
                Ok(()) => return Some((name, rom)),
                Err(e) => e,
            },
            Err(e) => e.to_string(),
        };
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("MegaFuse")
            .set_description(format!("{name}: {why}. Choose another."))
            .show();
    }
}
