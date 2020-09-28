//! A [BEP0015](https://www.bittorrent.org/beps/bep_0015.html)-conforming tracker
//! connection module, used primarily for
//! [Torrent::download](crate::Torrent::download)

use crate::error::TrackerError;
use crate::utils::randish_128;
use std::mem::size_of;
use std::net::UdpSocket;

/// The address typically used to bind a [UdpSocket] to for tracker connections
pub const TORRO_BIND_ADDR: &str = "127.0.0.1:7667";

/// The "magic constant" for connecting to the tracker. This is assumed to be the
/// bittorrent protocol id designation
const PROTOCOL_ID: i64 = 0x41727101980;

/// Calculates seconds for timeout according to the `15 * 2 ^ n` formula where
/// `n` is formatted as `tries` in this function signature, with an allowed range
/// of between 0 and 8
///
/// For extra reading, see the "
/// [Time outs](https://www.bittorrent.org/beps/bep_0015.html#time-outs)" section
/// of BEP0015 with examples of what `tries` to use
fn timeout_calc(tries: u8) -> u16 {
    assert!(!(tries > 8), "Timeouts can only be set as 0-8");

    15 * 2u16.pow(tries as u32) // TODO: make a rustc RFC for new `**` operator
}

/// Builds a connection request to be used to connect to the tracker in the form
/// of a `[u8; 16]` buffer, which may be converted into a `&[u8]` if needed
///
/// # BitTorrent Description
///
/// ```none
/// Before announcing or scraping, you have to obtain a connection ID.
///
///     Choose a random transaction ID.
///     Fill the connect request structure.
///     Send the packet.
///
/// connect request:
///
/// Offset  Size            Name            Value
/// 0       64-bit integer  protocol_id     0x41727101980 // magic constant
/// 8       32-bit integer  action          0 // connect
/// 12      32-bit integer  transaction_id
/// 16
/// ```
fn build_connect_req_buf(transaction_id: u32) -> [u8; 16] {
    let mut buf = [0x00; 16];

    buf[..size_of::<u64>()].copy_from_slice(&PROTOCOL_ID.to_be_bytes());
    buf[..size_of::<u32>()].copy_from_slice(&0u32.to_be_bytes());
    buf[..size_of::<u32>()].copy_from_slice(&transaction_id.to_be_bytes());

    buf
}

// TODO: tell user not to use this and make an automated higher-level func for all tracker info needs
// TODO: test
/// A connection request to a tracker, the first low-level exchange to and from
/// the client with the tracker
pub struct ConnectReq {
    /// Randomly-generated id that torro provides the tracker
    pub transaction_id: u32,
    /// The tracker-given connection id to reference for later use
    pub connection_id: u64,
}

impl ConnectReq {
    /// Sends a connection request from a given tracker `announce` URL and creates
    /// a new [ConnectReq] from it or returns a [TrackerError]
    ///
    /// `bind_addr` is typically just passed as the [TORRO_BIND_ADDR] constant,
    /// like so: `ConnectReq::send(TORRO_BIND_ADDR, something)`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use torro::tracker_udp::{TORRO_BIND_ADDR, ConnectReq};
    ///
    /// fn main() {
    ///     let connection_details = ConnectReq::send(
    ///         TORRO_BIND_ADDR,
    ///         "htp+t\\p:\\/tracker-url-here.co.biz".to_string()
    ///     ).unwrap();
    ///     
    ///     println!(
    ///         "Transaction ID: {}\nConnection ID: {}",
    ///         connection_details.transaction_id,
    ///         connection_details.connection_id
    ///     );
    /// }
    /// ```
    pub fn send(bind_addr: &'static str, announce: String) -> Result<Self, TrackerError> {
        let transaction_id = randish_128() as u32;
        let mut connection_buf = &build_connect_req_buf(transaction_id);

        let mut socket =
            UdpSocket::bind(bind_addr).map_err(|_| TrackerError::SocketBind(bind_addr))?;

        socket
            .send_to(connection_buf, &announce)
            .map_err(|_| TrackerError::SocketBind(bind_addr))?;

        // TODO: recieve inbound req and restructure to loop for timeouts with [timeout_calc]

        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that [timeout_calc] works correctly
    #[test]
    fn try_timeout_calc_norm() {
        assert_eq!(timeout_calc(0), 15);
        assert_eq!(timeout_calc(8), 3840);
    }

    /// Tests that [timeout_calc] correctly panics on invalidly entered integers
    #[test]
    #[should_panic]
    fn try_timeout_calc_panic() {
        timeout_calc(9);
    }

    /// Tests that [build_connect_req_buf] doesn't panic. This test is used as
    /// sometimes overflow safeguards are used but the function needs to convert
    /// (possibly very large) [u128] to [u32]
    #[test]
    fn build_connect_req_buf_stresstest() {
        for _ in 0..100 {
            build_connect_req_buf(randish_128() as u32);
        }
    }
}
