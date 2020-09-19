use climake::{Argument, CLIMake, DataType, PassedData, UsedArg};
use std::{path::PathBuf, process};
use torro::error::{BencodeError, TorrentCreationError, TorroError};
use torro::torrent::Torrent;

/// Prints given `msg` as an error then exits with code 1
fn error_exit(msg: String) -> ! {
    eprintln!("{}", msg);
    process::exit(1);
}

#[macro_export]
macro_rules! crate_version {
    () => {
        format!(
            "{}.{}.{}{}",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH"),
            option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
        )
    };
}

/// Entry function for magnet links passed in from user
fn do_maglink(_got_arg: UsedArg) {
    error_exit("Magnet links are currently not supported!".into());
}

/// Uses [Torrent::from_file] and handles any errors that may have occured
fn make_torrent(file: PathBuf) -> Torrent {
    match Torrent::from_file(file) {
        Ok(torrent) => torrent,
        Err(tl_err) => error_exit(match tl_err {
            TorroError::BadFileRead(_) => "IO error: could not read from torrent file".into(),
            TorroError::BencodeError(b_err) => format!("Bencode error: {}", match b_err {
                BencodeError::EmptyFile => "empty torrent file",
                BencodeError::InvalidInt(_) => "invalid integer given in torrent file",
                BencodeError::LeadingZeros(_) => "leading zeros given for an integer in torrent file",
                BencodeError::MultipleValues => "multiple values given for the top-level bencode",
                BencodeError::NoIntGiven(_) => "integer was perscribed but nothing was given",
                BencodeError::UnexpectedByte(_) => "unexpected byte was given",
                BencodeError::UnexpectedEOF => "torrent file ended prematurely, unexpected eof",
                BencodeError::NegativeZero(_) => "negative zero was given, zeros cannot be negative",
            }),
            TorroError::TorrentCreationError(tc_err) => format!("Torrent creation error: {}", match tc_err {
                TorrentCreationError::AnnounceWrongType => "`announce` key is the wrong type, expected bytestring",
                TorrentCreationError::FilesWrongType => "`files` key is the wrong type, expected list",
                TorrentCreationError::NameWrongType => "`announce` key is the wrong type, expected bytestring",
                TorrentCreationError::LengthWrongType => "`length` key is the wrong type, expected integer",
                TorrentCreationError::InfoWrongType => "`info` wrong type, expected dictionary",
                TorrentCreationError::PieceLengthWrongType => "`piece_length` wrong type, expected integer",
                TorrentCreationError::PiecesWrongType => "`pieces` wrong type, expected bytestring",
                TorrentCreationError::FileWrongType => "file in `files` wrong type, expected dict",
                TorrentCreationError::SubdirWrongType => "a subdirectory inside `path` key for a file is the wrong type, expected bytestring",
                TorrentCreationError::PathWrongType => "`path` key is the wrong type, expected bytestring",
                TorrentCreationError::BadUTF8String(_) => "badly formatted utf8 string given",
                TorrentCreationError::BothLengthFiles => "both `files` and `files` key was given, only one should be",
                TorrentCreationError::NoLengthFiles => "no `files` or `length` key given, needs one",
                TorrentCreationError::NoPiecesFound => "no `pieces` key given",
                TorrentCreationError::NoInfoFound => "no `info` key given",
                TorrentCreationError::NoAnnounceFound => "no `announce` key given",
                TorrentCreationError::NoNameFound => "no `name` key given",
                TorrentCreationError::NoPieceLengthFound => "no `pieces length` key given",
                TorrentCreationError::NoPathFound => "no `path` key given for a file inside `files` key",
                TorrentCreationError::NoTLDictionary => "no top-level dictionary given",
            }),
            _ => panic!() // do not cover errors for other parts of torro
        })
    }
}

/// Entry function for torrent files passed in from user
fn do_torrent_file(got_arg: UsedArg) {
    let torrent_file = match got_arg.passed_data {
        PassedData::File(file_vec) => {
            if file_vec.len() == 0 {
                error_exit("Please provide a file alongside the torrent argument!".into())
            // shouldn't happen
            } else if file_vec.len() > 1 {
                error_exit("Please provide only one file alongside the torrent argument!".into())
            } else {
                file_vec[0].clone()
            }
        }
        PassedData::None => {
            error_exit("Please provide a file alongside the torrent argument!".into())
        }
        _ => panic!(),
    };

    let _torrent = make_torrent(torrent_file);

    unimplemented!();
}

fn main() {
    let arg_maglink = Argument::new(
        &['m', 'l'],
        &["magnet"],
        Some("A magnet link to download from"),
        DataType::Text,
    )
    .unwrap();
    let arg_torrent = Argument::new(
        &['t'],
        &["torrent"],
        Some("A .torrent file to download from"),
        DataType::File,
    )
    .unwrap();

    let args = &[arg_maglink.clone(), arg_torrent.clone()];

    let cli = CLIMake::new(
        args,
        Some("A tiny cli-based torrent client."),
        Some(crate_version!()),
    )
    .unwrap();

    let mut maglink_buf = None;
    let mut torrent_buf = None;

    for used_arg in cli.parse() {
        if used_arg.argument == arg_maglink {
            maglink_buf = Some(used_arg)
        } else if used_arg.argument == arg_torrent {
            torrent_buf = Some(used_arg)
        }
    }

    if maglink_buf.is_some() {
        if torrent_buf.is_some() {
            eprintln!("Please give either a magnet link or a torrent file, not both!");
            process::exit(1);
        }

        do_maglink(maglink_buf.unwrap());
    } else if torrent_buf.is_some() {
        do_torrent_file(torrent_buf.unwrap());
    } else {
        eprintln!("Please provide either a magnet link or a torrent file!");
        process::exit(1);
    }
}
