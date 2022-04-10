use read_byte_slice::{ByteSliceIter, FallibleStreamingIterator};
use std::io::{self, Read};
use termcolor::{Color, ColorSpec, WriteColor};

/* TODO: can we build a lookup table from bytes to Braille characters at compile time?

const N: usize = 256;
const fn calculate_array() -> [char; N] {
    let mut res = [' '; N];
    let mut i = 0;
    while i < res.len() {
        res[i] = char::from_u32_unchecked(0x2800 + i as u32);
        i += 1;
    }
    res
}
const BRAILLE: [char; N] = calculate_array();
*/

/// Take a `u8` and return a `char` from the [Unicode Braille Patterns][u]
/// range representing the bits in that `u8`.
///
/// This is slightly more fiddly than I would like because Braille started
/// with only 6 total dots, so the Unicode codepoints are arranged following
/// that convention, with patterns including dots 7 and 8 tacked on at the end.
///
/// The Braille dots are numbered like so:
///
/// 1 4
/// 2 5
/// 3 6
/// 7 8
///
/// And the code points are ordered as `U+2800 + <binary bit pattern>`, where
/// the latter is simply counting from 0-255 in binary using the 8 dots as bits.
///
/// We want to produce output where the bits in the input `u8` map to dots
/// in the output like so:
///
/// 1 5
/// 2 6
/// 3 7
/// 4 8
///
/// To produce sensible looking results, then, we swap bit 4 in the input to
/// bit 7 in the output, and shift bits 5, 6, and 7 in the input rightwards one
/// bit in the output. Bits 1,2,3 and 8 remain unchanged.
///
/// [u]: https://unicode.org/charts/nameslist/n_2800.html
fn braille_byte(b: u8) -> char {
    let keep = b & 0b1000_0111;
    let b4 = b & 0b1000;
    let b567 = b & 0b0111_0000;
    let shuffled = keep | (b567 >> 1) | (b4 << 3);
    unsafe { char::from_u32_unchecked(0x2800 + u32::from(shuffled)) }
}

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
    W: WriteColor,
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
    pub fn hexdump<R, W>(self, r: R, mut w: W) -> io::Result<()>
        where
        R: Read,
        W: WriteColor,
    {
        let HexDump { elide_repeated } = self;
        let mut iter = ByteSliceIter::new(r, 16);
        let mut offset = 0;
        let mut prev = Vec::with_capacity(16);
        let mut elided_output = false;
        let mut bw = ColorSpec::new();
        bw.set_bg(Some(Color::Black)).set_fg(Some(Color::White));
        let mut wb = ColorSpec::new();
        wb.set_bg(Some(Color::White)).set_fg(Some(Color::Black));
        let colors = [bw, wb];
        let mut color_iter = colors.iter().cloned().cycle();
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
                    let byte_color = color_iter.next().unwrap();
                    let _ = w.set_color(&byte_color);
                    write!(w, "{}", braille_byte(*b))?;
                }
                let _ = w.reset();
                write!(w, " ")?;
            }
            // Skip the next color so byte colors will alternate line-by-line.
            let _ = color_iter.next();

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
