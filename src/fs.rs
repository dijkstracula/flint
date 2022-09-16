use packed_struct::prelude::*;

use std::{io::{Read, Write}, ops::{Index, Range, IndexMut}};

use crate::errors::*;

pub const MAGIC_BYTES: [u8; 4] = [0x42, 0x45, 0x56, 0x4f];

#[derive(PackedStruct)]
#[packed_struct(bit_numbering="msb0")]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Header {
    // 0x42, 0x45, 0x56, 0x4f
    #[packed_field(bits="0..=31")]
    magic: [u8; 4]
}

impl Header {
    pub fn new() -> Header {
        Header { magic: MAGIC_BYTES }
    }

    pub fn validate(&self) -> bool {
        self.magic == MAGIC_BYTES
    }
}

#[repr(C, align(512))]
pub struct Block([u8; 512]);

impl Block {
    pub fn new() -> Block {
        Block([0; 512])
    }

    pub fn read_from<R>(&mut self, r: &mut R) -> Result<(), Error> where R: Read {
        r.read_exact(&mut self.0)?;
        Ok(())
    }

    pub fn write_to<W>(&self, w: &mut W) -> Result<(), Error> where W: Write {
        w.write_all(&self.0)?;
        Ok(())
    }
}

impl Index<usize> for Block {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl Index<Range<usize>> for Block {
    type Output = [u8];
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.0[..][index]
    }
}

impl IndexMut<Range<usize>> for Block {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.0[..][index]
    }
}

#[cfg(test)]
mod tests {
    use crate::handle;

    #[test]
    fn fs_on_blk_dev() {
        assert!(handle::for_block_device("/dev/nonexistent").is_err());

        assert!(!handle::for_block_device("/dev/xvdb").is_err());

        let h = handle::for_block_device("/dev/xvdb");
        assert!(!h.is_err());

        let mut h = h.unwrap();

        let res = h.write("Writing to offset 1".as_bytes(), 1);
        let res = res.map_err(|e| { println!("{:?}", e); e} );
        assert!(res.is_ok());
    }

    #[test]
    fn fs_in_mem() {
        let h = handle::for_inmem(2);
        assert!(!h.is_err());

        let mut h = h.unwrap();

        let res = h.write("Writing to offset 1".as_bytes(), 1);
        assert!(res.is_ok());
    }
}