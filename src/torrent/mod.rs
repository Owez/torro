//! Contains main [Torrent] structure used as a "key" to interact with other parts
//! of torro

mod impl_bencode;
mod torrentstruct;

pub use impl_bencode::*;
pub use torrentstruct::*;
