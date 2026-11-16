//! Network play for MegaFuse (docs/NETPLAY.md): every peer runs the
//! whole game, only the pads cross the network; missing ones are
//! guessed and the game rolled back on a wrong guess.
//!
//! - [`transport`]: what the session needs of a network, as a trait that
//!   never blocks; [`matchbox`] (WebRTC data channels, through
//!   `crates/server`) and [`memory`] (one process, for tests) are two
//!   (both VacationDestination's).
//! - [`session`]: the host passes every seat's pads on to everyone, says
//!   which frames are confirmed and when a guess was wrong, seats peers
//!   coming in under way; the states' hashes are compared.
//! - [`wire`]: the messages as bytes.

#[cfg(feature = "matchbox")]
pub mod matchbox;
pub mod memory;
pub mod session;
pub mod transport;
pub mod wire;

pub use session::{Error, Session};
pub use transport::{PeerId, Role, Transport, TransportEvent};
