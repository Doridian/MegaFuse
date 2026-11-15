//! Network play for MegaFuse (docs/NETPLAY.md): every peer runs the
//! whole game, only the pads cross the network, in lockstep.
//!
//! - [`transport`]: what the session needs of a network, as a trait that
//!   never blocks; [`matchbox`] (WebRTC data channels, through
//!   `crates/server`) and [`memory`] (one process, for tests) are two
//!   (both VacationDestination's).
//! - [`session`]: the lockstep session: the host gathers each frame's
//!   pads and sends them to everyone; the states' hashes are compared.
//! - [`wire`]: the messages as bytes.

#[cfg(feature = "matchbox")]
pub mod matchbox;
pub mod memory;
pub mod session;
pub mod transport;
pub mod wire;

pub use session::{Error, Session};
pub use transport::{PeerId, Role, Transport, TransportEvent};
