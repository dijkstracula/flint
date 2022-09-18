use std::convert::TryInto;
use std::fs::File;
use std::io::{Read, Write, Seek, Cursor};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use linux_raw_sys::general::S_IFBLK;
use packed_struct::PackedStruct;
use rustix::fs::{fstat};
use rustix::io::Errno;

use crate::buffer::Buffer;
use crate::errors::Error;
use crate::fs::{Header, MAGIC_BYTES};

pub struct Handle<F: Read + Write + Seek> {
    backing_store: F,
}

fn blocks_required_for(s: usize) -> usize {
    if s == 0 {
        return 1; // TODO: ???
    }
    if s % 512 == 0 {
        return s / 512;
    }
    return s / 512 + 1;
}

fn block_offset_for(fpos: usize) -> (usize, usize) {
    let fpos = fpos + 512; /* For the header */
    (fpos & !((1 << 9) - 1), fpos & (1 << 9) - 1)
}
   
pub fn for_block_device(filename: &str) -> Result<Handle<File>, Error> {
    let path = Path::new(filename);

    let f = File::options()
        .read(true)
        .write(true)
        .custom_flags(linux_raw_sys::general::O_DIRECT.try_into().unwrap())
        .open(path)
        .map_err(|e| Errno::from_io_error(&e).expect("Invalid Errno???"))?;

    let stat = fstat(&f)?;
    if stat.st_mode & S_IFBLK == 0 {
        return Err(Error::Errno(Errno::NOTBLK.raw_os_error()));
    }

    Handle::new(f)
}

pub fn for_inmem(blocks: usize) -> Result<Handle<Cursor<Vec<u8>>>, Error> {
    Handle::new(Cursor::new(vec![0; blocks * 512]))
}


impl<F: Read + Write + Seek> Handle<F> {
    fn new(backing_store: F) -> Result<Handle<F>, Error> {
        let mut h = Handle { backing_store: backing_store };

        let mut buf = Buffer::new(1);
        buf.read_from(&mut h.backing_store)?;

        if buf[0..4] != MAGIC_BYTES {
            h.init()?;
        }

        Ok(h)
    }

    fn init(&mut self) -> Result<(), Error> {
        let blob = Header::new().pack()?;
        self.backing_store.write_all(&blob)?;
        Ok(())
    }

    pub fn write(&mut self, blob: &[u8], offset: usize) -> Result<usize, Error> {
        let mut buf = Buffer::new(blocks_required_for(blob.len()));

        let (nblock, offset) = block_offset_for(offset);
        self.backing_store.seek(std::io::SeekFrom::Start(nblock as u64))?;
        buf.read_from(&mut self.backing_store)?;

        buf[offset..offset + blob.len()].copy_from_slice(blob);

        buf.write_to(&mut self.backing_store)?;


        Ok(blob.len())
    }

}

#[cfg(test)]
mod tests {
    use crate::handle::*;

    #[test]
    fn test_block_offset_for() {
        let (b, o) = block_offset_for(0);
        assert_eq!(b, 512);
        assert_eq!(o, 0);


        let (b, o) = block_offset_for(1);
        assert_eq!(b, 512);
        assert_eq!(o, 1);
    }

    #[test]
    fn test_blocks_required_for() {
        assert_eq!(blocks_required_for(0), 1);
        assert_eq!(blocks_required_for(1), 1);
        assert_eq!(blocks_required_for(10), 1);
        assert_eq!(blocks_required_for(512), 1);

        assert_eq!(blocks_required_for(513), 2);
        assert_eq!(blocks_required_for(1000), 2);
        assert_eq!(blocks_required_for(1023), 2);
        assert_eq!(blocks_required_for(1024), 2);

        assert_eq!(blocks_required_for(1025), 3);
    }
}