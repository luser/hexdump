extern crate hexdump;

use hexdump::HexDump;
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io;

fn main() {
    let path = env::args_os().nth(1).expect("Usage: hexdump <filename>");
    match work(path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            ::std::process::exit(1);
        }
    }
}

fn work(path: OsString) -> io::Result<()> {
    let f = File::open(&path)?;
    let stdout = io::stdout();
    let handle = stdout.lock();
    HexDump::new()
        .elide_repeated(true)
        .hexdump(f, handle)
}
