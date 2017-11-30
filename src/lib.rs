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
///
/// To customize the output, use the [`HexDump`] builder.
///
/// [`HexDump`]: struct.HexDump.html
pub fn hexdump<R, W>(r: R, w: W) -> io::Result<()>
where
    R: Read,
    W: Write,
{
    HexDump::new().hexdump(r, w)
}


/// A builder for customizing the format of hex dump output.
#[derive(Default)]
pub struct HexDump {
    /// `true` if repeated lines of output should be replaced with `*`.
    elide_repeated: bool,
}

impl HexDump {
    /// Create a new `HexDump`. Call [`hexdump`] after calling other methods to customize
    /// the output format.
    ///
    /// [`hexdump`]: #method.hexdump
    pub fn new() -> HexDump {
        Default::default()
    }

    /// Set whether the output should elide repeated lines by replacing them with
    /// a single `*`. The default is `false`.
    pub fn elide_repeated(mut self, v: bool) -> HexDump {
        self.elide_repeated = v;
        self
    }

    /// Write a hexadecimal dump of bytes read from `r` to `w`.
    pub fn hexdump<R, W>(self, r: R, w: W) -> io::Result<()>
        where
        R: Read,
        W: Write,
    {
        let HexDump { elide_repeated } = self;
        let mut w = BufWriter::new(w);
        let mut iter = ByteSliceIter::new(r, 16);
        let mut offset = 0;
        let mut prev = Vec::with_capacity(16);
        let mut elided_output = false;
        while let Some(chunk) = iter.next()? {
            let current = offset;
            offset += chunk.len();
            if elide_repeated {
                if chunk == &prev[..] {
                    if !elided_output {
                        elided_output = true;
                        writeln!(w, "*")?;
                    }
                    continue;
                } else {
                    elided_output = false;
                }
            }
            write!(w, "{:08x}  ", current)?;
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
            for b in chunk.iter() {
                write!(w, "{}", printable(b))?
            }
            writeln!(w, "|")?;
            prev.clear();
            prev.extend_from_slice(chunk);
        }
        writeln!(w, "{:08x}", offset)?;
        Ok(())
    }
}
