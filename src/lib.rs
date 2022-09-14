use std::io::Seek;
use std::{fs::File, path::Path};

use rustix::fs::{fstat, FileExt};
use rustix::io::Errno;

use linux_raw_sys::general::S_IFBLK;

use packed_struct::prelude::*;

const MAGIC_BYTES: [u8; 4] = [0x42, 0x45, 0x56, 0x4f];

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

pub fn init(f: &mut File) -> Result<(), Errno> {
    let blob = Header::new().pack()
        .map_err(|_| Errno::BADMSG)?;

    f.write_all_at(&blob, 0)
        .map_err(|e| Errno::from_io_error(&e).expect("Invalid errno???"))
}

pub fn open(filename: &str) -> Result<File, Errno> {
    let path = Path::new(filename);
    let f = File::open(path)
        .map_err(|e| Errno::from_io_error(&e).expect("Invalid Errno???"))?;

    let stat = fstat(&f)?;

    if stat.st_mode & S_IFBLK == 0 {
        return Err(Errno::NOTBLK);
    }

    Ok(f)
}


#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        assert!(open("/dev/nonexistent").is_err());
        assert!(!open("/dev/xvdb").is_err());

        let f = open("/dev/xvdb");
        assert!(!f.is_err());

        assert!(!init(&mut f.unwrap()).is_err());
    }
}
