use hexdump::HexDump;
use std::{env, fs::File, io};
use termcolor::{BufferedStandardStream, ColorChoice, WriteColor};

fn main() -> io::Result<()> {
    let path = env::args_os().nth(1).expect("Usage: hexdump <filename>");
    let f = File::open(&path)?;
    let color = if atty::is(atty::Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };
    let mut stdout = BufferedStandardStream::stdout(color);
    let res = HexDump::new()
        .elide_repeated(true)
        .hexdump(f, &mut stdout);
    let _ = stdout.reset();
    res
}
