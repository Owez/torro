//! A simple example demonstrating parsing bencode and dealing with errors (cruedly)
//!
//! See [Bencode](bencode::Bencode), [bencode::parse] and
//! [BencodeError](torro::error::BencodeError) for more detailed documentation

use torro::bencode;

fn main() {
    let input_data = "l14:hello world31:this will be converted to bytes16:pretty cool, eh?e"
        .as_bytes()
        .to_vec(); // our inputted data as Vec<u8>

    match bencode::parse(input_data) {
        Ok(resulting_bencode) => println!(
            "All is well! Here's the resulting bencode:\n{:#?}",
            resulting_bencode
        ),
        Err(err) => eprintln!("Houston, we have a problem!: {:?}", err),
    };
}
