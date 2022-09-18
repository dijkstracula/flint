#![feature(allocator_api)]
#![feature(layout_for_ptr)]
#![feature(new_uninit)]
#![feature(pointer_is_aligned)]
#![feature(seek_stream_len)]

mod buffer;
pub mod errors;
pub mod fs;
pub mod handle;
