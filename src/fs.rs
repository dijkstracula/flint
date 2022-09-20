use packed_struct::prelude::*;

pub const MAGIC_BYTES: [u8; 4] = [0x42, 0x45, 0x56, 0x4f];

/// A Header needs to be present on block 0 of a block device.
#[derive(PackedStruct)]
#[packed_struct(bit_numbering = "msb0")]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Header {
    // 0x42, 0x45, 0x56, 0x4f
    #[packed_field(bits = "0..=31")]
    magic: [u8; 4],
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
    use crate::{buffer::Buffer, errors::Error, handle};

    use proptest::prelude::*;

    #[test]
    fn fs_on_blk_dev() {
        assert!(handle::for_block_device("/dev/nonexistent").is_err());

        let h = handle::for_block_device("/dev/xvdb");
        if h.is_err() {
            return;
        }

        let mut h = h.unwrap();

        let msg = "Writing to offset 1 in block 0".as_bytes();

        let mut buf = h.read_blocks(0, 1).unwrap();
        buf[1..(1 + msg.len())].copy_from_slice(&msg);

        let res = h.write(&buf, 0);
        assert!(res.is_ok());
    }

    #[test]
    fn fs_in_mem_rw() {
        let h = handle::for_inmem(1);
        assert!(!h.is_err());
        let mut h = h.unwrap();

        let expected_read = "Writing to offset 1 in block 0".as_bytes();

        let buf = h.read_blocks(0, 1);
        assert!(!buf.is_err());
        let mut buf = buf.unwrap();

        /* Write out some stuff into the buffer. */
        buf[1..(1 + expected_read.len())].copy_from_slice(&expected_read);

        let res = h.write(&buf, 0);
        assert!(res.is_ok());

        /* Read it back in. */
        let mut buf = Buffer::new_in_bytes(1);
        let res = h.read_into_buf(&mut buf, 0);
        assert!(res.is_ok());
        assert_eq!(&buf[1..1 + expected_read.len()], expected_read);
    }

    #[test]
    fn fs_oob() {
        let h = handle::for_inmem(1);
        assert!(!h.is_err());
        let mut h = h.unwrap();

        let mut buf = Buffer::new_in_bytes(1);

        let msg = "Writing to offset 1 in block 42".as_bytes();
        buf[1..(1 + msg.len())].copy_from_slice(&msg);

        let res = h.write(&buf, 42);
        assert!(!res.is_ok());
        assert_eq!(res.unwrap_err(), Error::AfterEOFAccess);
    }

    proptest! {
        #[test]
        fn test_read(
            devblocks in 10..256usize,
            bufblocks in 1..10usize) {
            let mut h = handle::for_inmem(devblocks).unwrap();

            // TODO: devblocks in 1..256usize, bufblocks in 1..devblocks ?
            let buf = h.read_blocks(0, bufblocks).unwrap();
            assert_eq!(buf.n_blocks, bufblocks);
        }

        #[test]
        fn test_rand_rw(
            devblocks in 10..256usize,
            bufblocks in 1..10usize) {
            let val = 0xdb;
            const BLOCK_BEGIN: usize = 0; /* TODO: how to vary this as part of the test and ensure it's in bounds? */

            let mut h = handle::for_inmem(devblocks).unwrap();

            // TODO: devblocks in 1..256usize, bufblocks in 1..devblocks ?
            let mut buf = Buffer::new_in_blocks(bufblocks);
            for v in buf.data.iter_mut() {
                *v = val;
            }
            h.write(&buf, BLOCK_BEGIN).unwrap();

            let buf = h.read_blocks(BLOCK_BEGIN, bufblocks).unwrap();
            for v in buf.data.iter() {
                assert_eq!(*v, val);
            }

        }

    }
}
