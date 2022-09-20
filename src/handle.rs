use std::cell::RefCell;
use std::convert::TryInto;
use std::fs::File;
use std::io::Seek;
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::path::Path;

use linux_raw_sys::general::S_IFBLK;
use packed_struct::PackedStruct;
use rustix::fs::fstat;

use crate::buffer::Buffer;
use crate::errors::Error;
use crate::fs::{Header, MAGIC_BYTES};

const BYTES_PER_DISK_BLOCK: usize = 1 << 20;

pub struct InMem(RefCell<Vec<u8>>);

impl FileExt for InMem {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        let begin = offset as usize;
        let end = begin + buf.len();

        buf.copy_from_slice(&self.0.borrow()[begin..end]);
        Ok(buf.len())
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        let begin = offset as usize;
        let end = begin + buf.len();
        self.0.borrow_mut()[begin..end].copy_from_slice(&buf);
        Ok(buf.len())
    }
}

// TODO: #[cfg(target_os = "linux")] but also squelch unused import warning
pub fn for_block_device(filename: &str) -> Result<Handle<File, BYTES_PER_DISK_BLOCK>, Error> {
    let path = Path::new(filename);

    let mut f = File::options()
        .read(true)
        .write(true)
        .custom_flags(linux_raw_sys::general::O_DIRECT.try_into().unwrap())
        .open(path)?;

    let stat = fstat(&f)?;
    if (stat.st_mode as u32) & S_IFBLK == 0 {
        return Err(Error::NotABlockDevice);
    }

    let sz = f.stream_len()?;
    Handle::new(f, sz)
}

pub fn for_inmem(blocks: usize) -> Result<Handle<InMem, 512>, Error> {
    let len = 512 + blocks * 512;
    Handle::new(InMem(RefCell::new(vec![0; len])), len as u64)
}

pub struct Handle<F, const BLOCK_SIZE: usize> {
    backing_store: F,
    len: u64,
}

impl<F: FileExt, const BLOCK_SIZE: usize> Handle<F, BLOCK_SIZE> {
    fn byte_offset_for(block: usize) -> u64 {
        (512 + block * BLOCK_SIZE) as u64
    }

    fn new(backing_store: F, len: u64) -> Result<Handle<F, BLOCK_SIZE>, Error> {
        let mut h = Handle { len, backing_store };
        let mut buf = Buffer::<BLOCK_SIZE>::new_in_bytes(1);

        h.backing_store.read_at(&mut buf.data, 0)?;

        if buf[0..4] != MAGIC_BYTES {
            h.init()?;
        }

        Ok(h)
    }

    fn init(&mut self) -> Result<(), Error> {
        let blob = Header::new().pack()?;

        let mut buf = Buffer::<BLOCK_SIZE>::new_in_bytes(1);
        buf[0..blob.len()].copy_from_slice(&blob);

        self.backing_store.write_all_at(&buf.data, 0)?;
        Ok(())
    }

    pub fn read_blocks(&mut self, blk_start: usize, n_blocks: usize) -> Result<Buffer<BLOCK_SIZE>, Error> {
        let mut buf = Buffer::new_in_blocks(n_blocks);
        self.read_into_buf(&mut buf, blk_start)?;
        Ok(buf)
    }

    pub fn read_into_buf(&mut self, buf: &mut Buffer<BLOCK_SIZE>, block: usize) -> Result<(), Error> {
        let byte_offset: u64 = Self::byte_offset_for(block);
        if byte_offset + buf.data.len() as u64 > self.len {
            return Err(Error::AfterEOFAccess);
        }

        let res = self
            .backing_store
            .read_exact_at(&mut buf.data, byte_offset)?;
        Ok(res)
    }

    pub fn write(&mut self, buf: &Buffer<BLOCK_SIZE>, block: usize) -> Result<(), Error> {
        let byte_offset: u64 = Self::byte_offset_for(block);
        if byte_offset + buf.data.len() as u64 > self.len {
            return Err(Error::AfterEOFAccess);
        }

        let res = self.backing_store.write_all_at(&buf.data, byte_offset)?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::handle::*;

    #[test]
    fn test_byte_offset_for() {
        assert_eq!(Handle::<InMem, 512>::byte_offset_for(0), 512);
        assert_eq!(Handle::<InMem, 512>::byte_offset_for(1), 512 + 512);
        assert_eq!(Handle::<InMem, 512>::byte_offset_for(3), 512 + (3 * 512));

        assert_eq!(Handle::<InMem, 4096>::byte_offset_for(0), 512);
        assert_eq!(Handle::<InMem, 4096>::byte_offset_for(1), 512 + 4096);
        assert_eq!(Handle::<InMem, 4096>::byte_offset_for(3), 512 + (3 * 4096));
    }
}
