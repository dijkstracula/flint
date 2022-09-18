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