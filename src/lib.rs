extern crate read_byte_slice;

use read_byte_slice::{ByteSliceIter, FallibleStreamingIterator};
use std::io::{self, BufWriter, Read, Write};

fn printable(b: &u8) -> char {
    let b = *b;
    if b > 0x1f && b < 0x7f {
        ::std::char::from_u32(b as u32).unwrap()
    } else {
        '.'
    }
}

/// Write a hexadecimal + ASCII dump of bytes read from `r` to `w`.
///
/// This function produces output equivalent to the default format of the `hd` tool,
/// consisting of an 8-byte offset in hex, 16 space-separated octets each in hex, split into
/// two groups of 8 octets each with two spaces between the groups, followed by an ASCII table
/// with each octet that is a printable ASCII character rendered as such, and other octets
/// rendered as an ASCII `FULL STOP` (`.`).
pub fn hexdump<R, W>(r: R, w: W) -> io::Result<()>
where
    R: Read,
    W: Write,
{
    let mut w = BufWriter::new(w);
    let mut iter = ByteSliceIter::new(r, 16);
    let mut offset = 0;
    while let Some(chunk) = iter.next()? {
        write!(w, "{:08x}  ", offset)?;
        for c in chunk.chunks(8) {
            for b in c {
                write!(w, "{:02x} ", b)?;
            }
            write!(w, " ")?;
        }
        // Pad out the rest of the line if necessary.
        for _ in 0..16 - chunk.len() {
            write!(w, "   ")?;
        }
        // Additional padding if the line is less than 8 bytes.
        if chunk.len() < 8 {
            write!(w, " ")?;
        }
        // Write the ASCII table.
        write!(w, "|")?;
        for b in chunk {
            write!(w, "{}", printable(b))?
        }
        writeln!(w, "|")?;
        offset += chunk.len();
    }
    writeln!(w, "{:08x}", offset)?;
    Ok(())
}
