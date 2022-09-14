use std::{fs::File, path::Path, io};
use std::os::unix::io::AsRawFd;

use rustix::fd;
use rustix::fs::fstat;
use rustix::io::Errno;

use linux_raw_sys::general;

pub fn open(filename: &str) -> Result<i32, Errno> {
    let path = Path::new(filename);
    let fd = File::open(path)
        .map(|f| f.as_raw_fd())
        .map_err(|e| Errno::from_io_error(&e).expect("Invalid Errno???"))?;

    let stat = fstat(fd::new(fd))?;

    Ok(fd)
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
