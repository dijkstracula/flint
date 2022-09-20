use std::{
    alloc::{Allocator, Global, Layout},
    ops::{Index, IndexMut, Range},
};

pub struct AlignedAlloc<const BLOCK_SIZE: usize>;

unsafe impl<const BLOCK_SIZE: usize> Allocator for AlignedAlloc<BLOCK_SIZE> {
    fn allocate(&self, layout: Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let layout = layout.align_to(BLOCK_SIZE).unwrap();
        Global.allocate(layout)
    }
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: Layout) {
        let layout = layout.align_to(BLOCK_SIZE).unwrap();
        Global.deallocate(ptr, layout)
    }
}

/// A Buffer marshals data to be written to or having been read from a block device.
pub struct Buffer<const BLOCK_SIZE: usize> {
    pub n_blocks: usize,
    pub data: Box<[u8], AlignedAlloc<BLOCK_SIZE>>,
}

impl<const BLOCK_SIZE: usize> Buffer<BLOCK_SIZE> {
    pub fn blocks_required_for(s: usize) -> usize {
        if s == 0 {
            return 1; // TODO: ???
        }
        if s % BLOCK_SIZE == 0 {
            return s / BLOCK_SIZE;
        }
        return s / BLOCK_SIZE + 1;
    }

    pub fn new_in_bytes(sz_bytes: usize) -> Buffer<BLOCK_SIZE> {
        let blocks_reqd = Buffer::<BLOCK_SIZE>::blocks_required_for(sz_bytes);
        Buffer::new_in_blocks(blocks_reqd)
    }

    pub fn new_in_blocks(sz_blocks: usize) -> Buffer<BLOCK_SIZE> {
        let sz_bytes = sz_blocks * BLOCK_SIZE;
        unsafe {
            let uninited: Box<[u8], AlignedAlloc<BLOCK_SIZE>> =
                Box::new_zeroed_slice_in(sz_bytes, AlignedAlloc).assume_init();
            Buffer {
                data: uninited,
                n_blocks: sz_blocks,
            }
        }
    }
}

impl<const BLOCK_SIZE: usize> Index<usize> for Buffer<BLOCK_SIZE> {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const BLOCK_SIZE: usize> Index<Range<usize>> for Buffer<BLOCK_SIZE> {
    type Output = [u8];
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.data[..][index]
    }
}

impl<const BLOCK_SIZE: usize> IndexMut<Range<usize>> for Buffer<BLOCK_SIZE> {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.data[..][index]
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::Buffer;

    #[test]
    fn test_alignment() {
        let b = Buffer::<512>::new_in_bytes(1024);
        assert_eq!(b.n_blocks, 2);
        assert_eq!(b.data.len(), 1024);
        assert!(b.data.as_ptr().is_aligned_to(512));

        let b = Buffer::<1024>::new_in_bytes(1024);
        assert_eq!(b.n_blocks, 1);
        assert_eq!(b.data.len(), 1024);
        assert!(b.data.as_ptr().is_aligned_to(1024));
        
        let b = Buffer::<4096>::new_in_bytes(1024);
        assert_eq!(b.n_blocks, 1);
        assert_eq!(b.data.len(), 4096);
        assert!(b.data.as_ptr().is_aligned_to(4096));

    }

    #[test]
    fn test_blocks_required_for() {
        assert_eq!(Buffer::<1048576>::blocks_required_for(0), 1);
        assert_eq!(Buffer::<1048576>::blocks_required_for(1), 1);
        assert_eq!(Buffer::<1048576>::blocks_required_for(10), 1);
        assert_eq!(Buffer::<1048576>::blocks_required_for(512), 1);

        assert_eq!(Buffer::<1048576>::blocks_required_for(1 << 20), 1);
        assert_eq!(Buffer::<1048576>::blocks_required_for(1 << 20 + 1), 2);
        assert_eq!(Buffer::<1048576>::blocks_required_for(1 << 21), 2);
        assert_eq!(Buffer::<1048576>::blocks_required_for(1 << 21 + 1), 4);

    }
}
