use std::{
    alloc::{Allocator, Global, Layout},
    ops::{Index, IndexMut, Range},
};


const BYTES_PER_BLOCK: usize = 512;

pub struct BufferAlloc;

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
    pub n_blocks: usize,
    pub data: Box<[u8], BufferAlloc>,
}

impl Buffer {
    pub fn blocks_required_for(s: usize) -> usize {
        if s == 0 {
            return 1; // TODO: ???
        }
        if s % 512 == 0 {
            return s / BYTES_PER_BLOCK;
        }
        return s / BYTES_PER_BLOCK + 1;
    }

    pub fn new_in_bytes(sz_bytes: usize) -> Buffer {
        let blocks_reqd = Buffer::blocks_required_for(sz_bytes);
        Buffer::new_in_blocks(blocks_reqd)
    }

    pub fn new_in_blocks(sz_blocks: usize) -> Buffer {
        let sz_bytes = sz_blocks * BYTES_PER_BLOCK;
        unsafe {
            let uninited: Box<[u8], BufferAlloc> =
                Box::new_zeroed_slice_in(sz_bytes, BufferAlloc).assume_init();
            Buffer {
                data: uninited,
                n_blocks: sz_blocks,
            }
        }
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
    use crate::buffer::{Buffer, BYTES_PER_BLOCK};

    #[test]
    fn test_alignment() {
        const BYTES: usize = 14;
        let b = Buffer::new_in_bytes(BYTES);

        assert_eq!(b.n_blocks, 1);
        assert_eq!(b.data.len(), BYTES_PER_BLOCK);
        assert!(b.data.as_ptr().is_aligned_to(BYTES_PER_BLOCK));
    }

    #[test]
    fn test_blocks_required_for() {
        assert_eq!(Buffer::blocks_required_for(0), 1);
        assert_eq!(Buffer::blocks_required_for(1), 1);
        assert_eq!(Buffer::blocks_required_for(10), 1);
        assert_eq!(Buffer::blocks_required_for(512), 1);

        assert_eq!(Buffer::blocks_required_for(513), 2);
        assert_eq!(Buffer::blocks_required_for(1000), 2);
        assert_eq!(Buffer::blocks_required_for(1023), 2);
        assert_eq!(Buffer::blocks_required_for(1024), 2);

        assert_eq!(Buffer::blocks_required_for(1025), 3);
    }

}
