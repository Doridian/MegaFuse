//! The session's messages as bytes: a tag, then the fields,
//! little-endian.  `VERSION` changes with the format (or with what makes
//! the game run differently): peers of different versions refuse each
//! other.

/// The protocol's version.
pub const VERSION: u16 = 2;

/// The biggest message: 16 KiB, which browsers' data channels all take
/// (the session's are far smaller).
pub const FRAME: usize = 16 * 1024;

/// The most runs a [`Message::History`] carries (10 bytes each).
pub const RUNS: usize = 1000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// A client to the host: its version and its ROM's CRC-32.
    Hello { version: u16, rom: u32 },
    /// The host to a client: the seat kept for it (pad 0-3), how many
    /// seats there are, the input delay, and the frames at the start
    /// whose pads are all 0.  The seat is the client's once it has
    /// caught up ([`Message::Ready`], [`Message::Seat`]).
    Welcome {
        slot: u8,
        players: u8,
        delay: u8,
        quiet: u64,
    },
    /// The host to a client: not let in, and why.
    Refuse(String),
    /// A seat's pad in a frame: a client's own to the host, and the host
    /// passing on everyone's (its own, the empty seats', the others').
    Input { frame: u64, slot: u8, pad: u16 },
    /// The host to a client coming in: the frames' pads from `from` on,
    /// as runs of (frames, the four pads).
    History {
        from: u64,
        runs: Vec<(u16, [u16; 4])>,
    },
    /// A client to the host: its state's hash after a frame.
    Hash { frame: u64, hash: u32 },
    /// The host to everyone: the states parted at a frame.
    Desync { frame: u64 },
    /// A client to the host: it has caught up, its seat can be its own.
    Ready,
    /// The host to everyone: a seat's pads are its player's from `from`.
    Seat { slot: u8, from: u64 },
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
            Message::Welcome {
                slot,
                players,
                delay,
                quiet,
            } => {
                b.extend_from_slice(&[2, *slot, *players, *delay]);
                b.extend_from_slice(&quiet.to_le_bytes());
            }
            Message::Refuse(why) => {
                b.push(3);
                b.extend_from_slice(why.as_bytes());
            }
            Message::Input { frame, slot, pad } => {
                b.push(4);
                b.extend_from_slice(&frame.to_le_bytes());
                b.push(*slot);
                b.extend_from_slice(&pad.to_le_bytes());
            }
            Message::History { from, runs } => {
                b.push(5);
                b.extend_from_slice(&from.to_le_bytes());
                for (n, pads) in runs {
                    b.extend_from_slice(&n.to_le_bytes());
                    for p in pads {
                        b.extend_from_slice(&p.to_le_bytes());
                    }
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
            Message::Ready => b.push(8),
            Message::Seat { slot, from } => {
                b.extend_from_slice(&[9, *slot]);
                b.extend_from_slice(&from.to_le_bytes());
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
            1 => Message::Hello {
                version: u16_at(0)?,
                rom: u32_at(2)?,
            },
            2 => Message::Welcome {
                slot: *rest.first()?,
                players: *rest.get(1)?,
                delay: *rest.get(2)?,
                quiet: u64_at(3)?,
            },
            3 => Message::Refuse(String::from_utf8_lossy(rest).into_owned()),
            4 => Message::Input {
                frame: u64_at(0)?,
                slot: *rest.get(8)?,
                pad: u16_at(9)?,
            },
            5 => {
                let body = rest.get(8..)?;
                if body.len() % 10 != 0 {
                    return None;
                }
                let runs = body
                    .chunks(10)
                    .map(|c| {
                        let w = |k: usize| u16::from_le_bytes([c[k], c[k + 1]]);
                        (w(0), [w(2), w(4), w(6), w(8)])
                    })
                    .collect();
                Message::History {
                    from: u64_at(0)?,
                    runs,
                }
            }
            6 => Message::Hash {
                frame: u64_at(0)?,
                hash: u32_at(8)?,
            },
            7 => Message::Desync { frame: u64_at(0)? },
            8 => Message::Ready,
            9 => Message::Seat {
                slot: *rest.first()?,
                from: u64_at(1)?,
            },
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
            Message::Hello {
                version: 1,
                rom: 0xDEAD_BEEF,
            },
            Message::Welcome {
                slot: 2,
                players: 4,
                delay: 3,
                quiet: 240,
            },
            Message::Refuse("full".into()),
            Message::Input {
                frame: 1 << 40,
                slot: 3,
                pad: 0x80,
            },
            Message::History {
                from: 241,
                runs: vec![(5, [1, 2, 3, 0xFFFF]), (1, [0; 4])],
            },
            Message::History {
                from: 1,
                runs: vec![],
            },
            Message::Hash {
                frame: 9,
                hash: 12345,
            },
            Message::Desync { frame: 11 },
            Message::Ready,
            Message::Seat { slot: 1, from: 500 },
        ] {
            assert_eq!(Message::decode(&m.encode()), Some(m));
        }
        assert_eq!(Message::decode(&[]), None);
        assert_eq!(Message::decode(&[4, 1, 2]), None);
    }
}
