use packed_struct::prelude::*;


pub const MAGIC_BYTES: [u8; 4] = [0x42, 0x45, 0x56, 0x4f];

/// A Header needs to be present on block 0 of a block device.
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
        // TODO: maybe this should consume a Block instead?
        self.magic == MAGIC_BYTES
    }
}


#[cfg(test)]
mod tests {
    use crate::{handle, buffer::Buffer};

    /* 
    #[test]
    fn fs_on_blk_dev() {
        assert!(handle::for_block_device("/dev/nonexistent").is_err());

        assert!(!handle::for_block_device("/dev/xvdb").is_err());

        let h = handle::for_block_device("/dev/xvdb");
        assert!(!h.is_err());
        let mut h = h.unwrap();

        let msg = "Writing to offset 1 in block 0".as_bytes();
        let mut buf = Buffer::new(1);
        buf[1..(1 + msg.len())].copy_from_slice(&msg);

        let res = h.write(&buf, 0);
        assert!(res.is_ok());
    }
    */

    #[test]
    fn fs_in_mem() {
        let h = handle::for_inmem(1);
        assert!(!h.is_err());
        let mut h = h.unwrap();
        
        let mut buf = Buffer::new(1);

        let msg = "Writing to offset 1 in block 0".as_bytes();
        buf[1..(1 + msg.len())].copy_from_slice(&msg);

        let res = h.write(&buf, 0);
        assert!(res.is_ok());

        let msg = "Writing to offset 1 in block 42".as_bytes();
        buf[1..(1 + msg.len())].copy_from_slice(&msg);

        let res = h.write(&buf, 42);
        assert!(!res.is_ok());
        
    }
}