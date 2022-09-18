use std::{alloc::{Layout, Allocator, Global}, io::{Read, Write}, ops::{Index, IndexMut, Range}};

use crate::errors::Error;

const BYTES_PER_BLOCK: usize = 512;

struct BufferAlloc;

unsafe impl Allocator for BufferAlloc {
    fn allocate(&self, layout: Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let layout = layout.align_to(BYTES_PER_BLOCK).unwrap();
        Global.allocate(layout)
    }
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: Layout) {
        let layout = layout.align_to(BYTES_PER_BLOCK).unwrap();
        Global.deallocate(ptr, layout)
    }
}

/// A Buffer marshals data to be written to or having been read from a block device.
pub struct Buffer {
    n_blocks: usize,
    data: Box<[u8], BufferAlloc>,
}

impl Buffer {
    pub fn new(blocks: usize) -> Buffer {
        unsafe {
            let uninited: Box<[u8], BufferAlloc> = 
                Box::new_uninit_slice_in(blocks * BYTES_PER_BLOCK, BufferAlloc).assume_init();
            Buffer {
                data: uninited,
                n_blocks: blocks
            }
        }
    }

    // TODO: maybe just consume a Handle
    pub fn read_from<R>(&mut self, r: &mut R) -> Result<(), Error> where R: Read {
        r.read_exact(&mut self.data)?;
        Ok(())
    }

    pub fn write_to<W>(&self, w: &mut W) -> Result<(), Error> where W: Write {
        // TODO: mark dirty blocks and write those out.
        // TODO: at some point we would decide "enough blocks are dirty so we should
        // redundantly write clean blocks to minimise write calls".  What's a good
        // metric?
        w.write_all(&self.data)?;
        Ok(())
    }
}

impl Index<usize> for Buffer {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl Index<Range<usize>> for Buffer {
    type Output = [u8];
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.data[..][index]
    }
}

impl IndexMut<Range<usize>> for Buffer {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.data[..][index]
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use crate::{buffer::{Buffer, BYTES_PER_BLOCK}};

    #[test]
    fn test_alignment() {
        const BLOCKS: usize = 14;
        let b = Buffer::new(BLOCKS);

        assert_eq!(b.n_blocks, BLOCKS);
        assert_eq!(b.data.len(), BYTES_PER_BLOCK * BLOCKS);
        assert!(b.data.as_ptr().is_aligned_to(512));
    }
}