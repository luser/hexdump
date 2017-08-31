extern crate fallible_streaming_iterator;

use fallible_streaming_iterator::FallibleStreamingIterator;
use std::cmp;
use std::io::{self, BufRead, BufReader, Read, Write};

fn printable(b: &u8) -> char {
    let b = *b;
    if b > 0x1f && b < 0x7f {
        ::std::char::from_u32(b as u32).unwrap()
    } else {
        '.'
    }
}

struct ByteSliceIter<R>
where
    R: Read,
{
    inner: BufReader<R>,
    buf: Vec<u8>,
}

impl<R> ByteSliceIter<R>
where
    R: Read,
{
    pub fn new(inner: R, slice_len: usize) -> ByteSliceIter<R> {
        ByteSliceIter {
            inner: BufReader::new(inner),
            buf: Vec::with_capacity(slice_len),
        }
    }
}

impl<'a, R> FallibleStreamingIterator for ByteSliceIter<R>
where
    R: Read,
{
    type Item = [u8];
    type Error = io::Error;

    fn advance(&mut self) -> Result<(), io::Error> {
        if self.buf.len() > 0 {
            self.inner.consume(self.buf.len());
            self.buf.clear();
        }
        let buf = self.inner.fill_buf()?;
        let cap = self.buf.capacity();
        self.buf.extend_from_slice(
            &buf[..cmp::min(buf.len(), cap)],
        );
        Ok(())
    }

    fn get(&self) -> Option<&[u8]> {
        if self.buf.len() > 0 {
            Some(self.buf.as_slice())
        } else {
            None
        }
    }
}

pub fn hexdump<R, W>(r: R, mut w: W) -> io::Result<()>
where
    R: Read,
    W: Write,
{
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
        // Pad out the rest of the line.
        for _ in 0..16 - chunk.len() {
            write!(w, "   ")?;
        }
        if chunk.len() < 8 {
            write!(w, " ")?;
        }
        write!(w, "|{}|", chunk.iter().map(printable).collect::<String>())?;
        writeln!(w, "")?;
        offset += chunk.len();
    }
    writeln!(w, "{:08x}", offset)?;
    Ok(())
}
