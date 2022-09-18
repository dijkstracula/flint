use std::convert::TryInto;
use std::fs::File;
use std::io::{Read, Write, Seek, Cursor};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use linux_raw_sys::general::S_IFBLK;
use packed_struct::PackedStruct;
use rustix::fs::{fstat};

use crate::buffer::Buffer;
use crate::errors::Error;
use crate::fs::{Header, MAGIC_BYTES};

pub struct Handle<F: Read + Write + Seek> {
    backing_store: F,
}

fn byte_offset_for(block: usize) -> u64 {
    512 + block as u64 * 512
}

pub fn for_block_device(filename: &str) -> Result<Handle<File>, Error> {
    let path = Path::new(filename);

    let f = File::options()
        .read(true)
        .write(true)
        .custom_flags(linux_raw_sys::general::O_DIRECT.try_into().unwrap())
        .open(path)?;

    let stat = fstat(&f)?;
    if stat.st_mode & S_IFBLK == 0 {
        return Err(Error::NotABlockDevice);
    }

    Handle::new(f)
}

pub fn for_inmem(blocks: usize) -> Result<Handle<Cursor<Vec<u8>>>, Error> {
    Handle::new(Cursor::new(vec![0; 512 + blocks * 512]))
}


impl<F: Read + Write + Seek> Handle<F> {
    fn new(backing_store: F) -> Result<Handle<F>, Error> {
        let mut h = Handle { backing_store: backing_store };
        let mut buf = Buffer::new(1);

        h.backing_store.rewind()?;
        h.backing_store.read(&mut buf.data)?;

        if buf[0..4] != MAGIC_BYTES {
            h.init()?; 
        }

        Ok(h)
    }

    fn init(&mut self) -> Result<(), Error> {
        let blob = Header::new().pack()?;

        let mut buf = Buffer::new(1);
        buf[0..blob.len()].copy_from_slice(&blob);

        self.backing_store.rewind()?;
        self.backing_store.write_all(&buf.data)?;
        Ok(())
    }

    pub fn read(&mut self, buf: &mut Buffer, block: usize) -> Result<(), Error> {
        let byte_offset: u64 = byte_offset_for(block);

        self.backing_store.seek(std::io::SeekFrom::Start(byte_offset))?;
        let res = self.backing_store.read_exact(&mut buf.data)?;
        Ok(res)
    }

    pub fn write(&mut self, buf: &Buffer, block: usize) -> Result<(), Error> {
        let byte_offset: u64 = byte_offset_for(block);

        self.backing_store.seek(std::io::SeekFrom::Start(byte_offset))?;
        let res = self.backing_store.write_all(&buf.data)?;
        Ok(res)
    }

}

#[cfg(test)]
mod tests {
    use crate::handle::*;

    #[test]
    fn test_byte_offset_for() {
        assert_eq!(byte_offset_for(0), 512);
        assert_eq!(byte_offset_for(1), 1024);
    }

}