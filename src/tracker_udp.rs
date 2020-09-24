//! A [BEP0015](https://www.bittorrent.org/beps/bep_0015.html)-conforming tracker
//! connection module, used primarily for
//! [Torrent::download](crate::torrent::Torrent::download)

use crate::utils::randish_128;
use std::mem::size_of;
use std::net::UdpSocket;

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
fn build_connect_req() -> [u8; 16] {
    let transaction_id = randish_128() as u32;

    let mut buf = [0x00; 16];

    buf[..size_of::<u64>()].copy_from_slice(&PROTOCOL_ID.to_be_bytes());
    buf[..size_of::<u32>()].copy_from_slice(&0u32.to_be_bytes());
    buf[..size_of::<u32>()].copy_from_slice(&transaction_id.to_be_bytes());

    buf
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

    /// Tests that [build_connect_req] doesn't panic. This test is used as
    /// sometimes overflow safeguards are used but the function needs to convert
    /// (possibly very large) [u128] to [u32]
    #[test]
    fn build_connect_req_stresstest() {
        for _ in 0..100 {
            build_connect_req();
        }
    }
}
