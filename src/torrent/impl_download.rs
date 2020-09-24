//! Contains Torrent::download-related functionality
//!
//! NOTE: Currently used as a placeholder module with many `unimplemented!()`

use crate::error::TorroError;
use crate::torrent::Torrent;
// use crate::tracker_udp; // TODO: import

impl Torrent {
    /// Downloads given torrent to the defined file/directory ([Torrent::name])
    ///
    /// If an error is encountered, it will be a
    /// [TrackerError] wrapped inside of
    /// [TorroError::TrackerError](TorroError::TrackerError)
    pub fn download(&self) -> Result<(), TorroError> {
        let tracker_info = self.get_tracker_info();

        unimplemented!(); // TODO: finish
    }

    /// Gets tracker infomation from [torro::tracker_udp]
    fn get_tracker_info(&self) -> ! {
        // TODO: find return type and tracker udp module name
        unimplemented!();
    }
}
