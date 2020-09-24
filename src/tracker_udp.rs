//! A [BEP0015](https://www.bittorrent.org/beps/bep_0015.html)-conforming tracker
//! connection module, used primarily for
//! [Torrent::download](crate::torrent::Torrent::download)

use crate::utils::randish_128;
use std::net::UdpSocket;

/// Calculates seconds for timeout according to the `15 * 2 ^ n` formula where
/// `n` is formatted as `tries` in this function signature, with an allowed range
/// of between 0 and 8
///
/// For extra reading, see the "
/// [Time outs](https://www.bittorrent.org/beps/bep_0015.html#time-outs)" section
/// of BEP0015 with examples of what `tries` to use
fn timeout_calc(tries: u8) -> u16 {
    assert!(tries > 8, "Timeouts can only be set as 0-8");

    15 * 2u16.pow(tries as u32) // TODO: make a rustc RFC for new `**` operator
}

/// Builds a connection request to be used to connect to the tracker in the form
/// of a `&[u8]`. The `'tcr` lifetime represents the "tracker connect(ion) request"
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
fn build_connect_req<'tcr>() -> &'tcr [u8] {
    let transaction_id = randish_128() as u32;

    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that [timeout_calc] works correctly
    #[test]
    fn try_timeout_calc() {
        assert_eq!(timeout_calc(0), 15);
        assert_eq!(timeout_calc(8), 3840);
    }
}
