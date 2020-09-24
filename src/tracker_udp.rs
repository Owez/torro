//! A [BEP0015](https://www.bittorrent.org/beps/bep_0015.html)-conforming tracker
//! connection module, used primarily for
//! [Torrent::download](crate::torrent::Torrent::download)

use std::net::UdpSocket;
use crate::utils::randish_128;

/// Calculates seconds for timeout according to the `15 * 2 ^ n` formula where
/// `n` is formatted as `tries` in this function signature
/// 
/// For extra reading, see the "
/// [Time outs](https://www.bittorrent.org/beps/bep_0015.html#time-outs)" section
/// of BEP0015 with examples of what `tries` to use
fn timeout_calc(tries: u8) -> u16 {
    15 * 2 ^ tries as u16
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
