//! The lockstep session without a game: peers over the in-memory
//! transport, each on a thread, feed it pads made up from the frame and
//! their seat; every peer has to get the same pads for every frame, each
//! player's own a `delay` after it gave them.

use std::sync::mpsc;

use net::Session;
use net::memory::Hub;

const FRAMES: u64 = 300;
const DELAY: u64 = 3;

/// What a player presses at a frame.
fn pressed(slot: usize, frame: u64) -> u16 {
    ((frame * 7 + slot as u64 * 13) % 256) as u16
}

#[test]
fn peers_get_the_same_frames() {
    let players = 3;
    let hub = Hub::new();
    let (tx, rx) = mpsc::channel();
    let mut threads = Vec::new();
    for k in 0..players {
        let t = hub.transport();
        let tx = tx.clone();
        threads.push(std::thread::spawn(move || {
            let mut s = if k == 0 {
                Session::host(t, "room", players, DELAY, 0xABCD)
            } else {
                std::thread::sleep(std::time::Duration::from_millis(20));
                Session::join(t, "room", 0xABCD)
            }
            .expect("in the game");
            let (slot, n) = s.seat();
            assert_eq!(n, players);
            let mut got = Vec::new();
            for f in 1..=FRAMES {
                let hash = f.is_multiple_of(60).then_some(f as u32);
                got.push(s.pads(f, pressed(slot, f), hash).expect("a frame"));
            }
            tx.send((slot, got)).expect("sent");
        }));
    }
    drop(tx);
    let mut runs: Vec<(usize, Vec<[u16; 4]>)> = rx.iter().collect();
    for t in threads {
        t.join().expect("peer ran");
    }
    runs.sort_by_key(|r| r.0);
    assert_eq!(runs.iter().map(|r| r.0).collect::<Vec<_>>(), vec![0, 1, 2]);
    for (_, got) in &runs[1..] {
        assert_eq!(got, &runs[0].1, "every peer gets the same pads");
    }
    for (i, pads) in runs[0].1.iter().enumerate() {
        let f = i as u64 + 1;
        for (slot, &pad) in pads.iter().enumerate().take(players) {
            let want = if f <= DELAY { 0 } else { pressed(slot, f - DELAY) };
            assert_eq!(pad, want, "frame {f}, player {slot}");
        }
        assert_eq!(pads[3], 0);
    }
}

#[test]
fn another_rom_is_refused() {
    let hub = Hub::new();
    let host = hub.transport();
    let client = hub.transport();
    let h = std::thread::spawn(move || Session::host(host, "room", 2, DELAY, 1).map(|_| ()));
    std::thread::sleep(std::time::Duration::from_millis(20));
    let joined = Session::join(client, "room", 2);
    assert!(matches!(joined, Err(net::Error::Refused(_))), "refused");
    drop(h);
}
