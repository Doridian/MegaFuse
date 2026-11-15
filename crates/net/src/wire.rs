//! The session's messages as bytes: a tag, then the fields,
//! little-endian.  `VERSION` changes with the format (or with what makes
//! the game run differently): peers of different versions refuse each
//! other.

/// The protocol's version.
pub const VERSION: u16 = 1;

/// The biggest message: 16 KiB, which browsers' data channels all take
/// (the session's are far smaller).
pub const FRAME: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// A client to the host: its version and its ROM's CRC-32.
    Hello { version: u16, rom: u32 },
    /// The host to a client: its player (pad 0-3), how many there are,
    /// the input delay (frames).
    Welcome { slot: u8, players: u8, delay: u8 },
    /// The host to a client: not let in, and why.
    Refuse(String),
    /// A client to the host: its pad in a frame.
    Input { frame: u64, pad: u16 },
    /// The host to everyone: a frame's four pads, the same for all.
    Frame { frame: u64, pads: [u16; 4] },
    /// A client to the host: its state's hash after a frame.
    Hash { frame: u64, hash: u32 },
    /// The host to everyone: the states parted at a frame.
    Desync { frame: u64 },
}

impl Message {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut b = Vec::new();
        match self {
            Message::Hello { version, rom } => {
                b.push(1);
                b.extend_from_slice(&version.to_le_bytes());
                b.extend_from_slice(&rom.to_le_bytes());
            }
            Message::Welcome { slot, players, delay } => b.extend_from_slice(&[2, *slot, *players, *delay]),
            Message::Refuse(why) => {
                b.push(3);
                b.extend_from_slice(why.as_bytes());
            }
            Message::Input { frame, pad } => {
                b.push(4);
                b.extend_from_slice(&frame.to_le_bytes());
                b.extend_from_slice(&pad.to_le_bytes());
            }
            Message::Frame { frame, pads } => {
                b.push(5);
                b.extend_from_slice(&frame.to_le_bytes());
                for p in pads {
                    b.extend_from_slice(&p.to_le_bytes());
                }
            }
            Message::Hash { frame, hash } => {
                b.push(6);
                b.extend_from_slice(&frame.to_le_bytes());
                b.extend_from_slice(&hash.to_le_bytes());
            }
            Message::Desync { frame } => {
                b.push(7);
                b.extend_from_slice(&frame.to_le_bytes());
            }
        }
        b
    }

    #[must_use]
    pub fn decode(b: &[u8]) -> Option<Message> {
        let (&tag, rest) = b.split_first()?;
        let u16_at = |k: usize| Some(u16::from_le_bytes(rest.get(k..k + 2)?.try_into().ok()?));
        let u32_at = |k: usize| Some(u32::from_le_bytes(rest.get(k..k + 4)?.try_into().ok()?));
        let u64_at = |k: usize| Some(u64::from_le_bytes(rest.get(k..k + 8)?.try_into().ok()?));
        Some(match tag {
            1 => Message::Hello { version: u16_at(0)?, rom: u32_at(2)? },
            2 => Message::Welcome { slot: *rest.first()?, players: *rest.get(1)?, delay: *rest.get(2)? },
            3 => Message::Refuse(String::from_utf8_lossy(rest).into_owned()),
            4 => Message::Input { frame: u64_at(0)?, pad: u16_at(8)? },
            5 => Message::Frame {
                frame: u64_at(0)?,
                pads: [u16_at(8)?, u16_at(10)?, u16_at(12)?, u16_at(14)?],
            },
            6 => Message::Hash { frame: u64_at(0)?, hash: u32_at(8)? },
            7 => Message::Desync { frame: u64_at(0)? },
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Message;

    #[test]
    fn round_trips() {
        for m in [
            Message::Hello { version: 1, rom: 0xDEAD_BEEF },
            Message::Welcome { slot: 2, players: 4, delay: 3 },
            Message::Refuse("full".into()),
            Message::Input { frame: 1 << 40, pad: 0x80 },
            Message::Frame { frame: 7, pads: [1, 2, 3, 0xFFFF] },
            Message::Hash { frame: 9, hash: 12345 },
            Message::Desync { frame: 11 },
        ] {
            assert_eq!(Message::decode(&m.encode()), Some(m));
        }
        assert_eq!(Message::decode(&[]), None);
        assert_eq!(Message::decode(&[5, 1, 2]), None);
    }
}
