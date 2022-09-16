use std::fs::File;
use std::io::{Read, Write, Seek, Cursor};
use std::path::Path;

use linux_raw_sys::general::S_IFBLK;
use packed_struct::PackedStruct;
use rustix::fs::{fstat};
use rustix::io::Errno;

use crate::errors::Error;
use crate::fs::{Header, MAGIC_BYTES};

pub struct Handle<F: Read + Write + Seek> {
    backing_store: F,
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

        let mut magic: [u8; 4] = [0; 4]; // TODO: init

        let n = h.backing_store.read(&mut magic)?;
        if n != 4 {
            return Err(Error::Errno(Errno::IO.raw_os_error()));
        }
        if magic != MAGIC_BYTES {
            h.init()?;
        }

        Ok(h)
    }

    fn init(&mut self) -> Result<(), Error> {
        let blob = Header::new().pack()?;
        self.backing_store.write_all(&blob)?;
        Ok(())
    }

    pub fn write(&mut self, blob: &[u8], posn: usize) -> Result<usize, Error> {
        let (nblock, offset) = block_offset_for(posn);
        let mut block: [u8; 512] = [0; 512];

        self.backing_store.seek(std::io::SeekFrom::Start(nblock as u64))?;

        /* XXX: larger than one block; straddling one block! */
        let n = self.backing_store.read(&mut block)?;
        if n != 512 {
            return Err(Error::Errno(Errno::IO.raw_os_error()));
        }

        block[offset..offset + blob.len()].copy_from_slice(blob);

        self.backing_store.write_all(&block)?;

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
}