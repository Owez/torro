//! Contains Torrent::download-related functionality
//!
//! NOTE: Currently used as a placeholder module with many `unimplemented!()`

use crate::torrent::Torrent;
use crate::error::TorroError;

impl Torrent {
    /// Downloads given torrent to the defined file/directory ([Torrent::file_path])
    pub fn download(&self) -> Result<(), TorroError> {
        let tracker_info = self.get_tracker_info();

        unimplemented!(); // TODO: finish
    }
    
    /// Gets tracker infomation from [torro::tracker_udp] (or similar)
    fn get_tracker_info(&self) -> ! {
        // TODO: find return type and tracker udp module name
        unimplemented!();
    }
}
