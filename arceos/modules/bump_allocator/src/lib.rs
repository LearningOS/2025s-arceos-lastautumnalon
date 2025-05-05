#![no_std]

use core::sync::atomic::AtomicUsize;
use core::alloc::Layout;
use core::ptr::NonNull;

use allocator::{BaseAllocator, ByteAllocator, PageAllocator, AllocResult, AllocError};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,   // 起始地址
    end: usize,     // 结束地址

    b_pos: usize,   // byte allocator bump position
    p_pos: usize,   // page allocator bump position

    byte_alloc_count: AtomicUsize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE>{
    /// Creates a new empty ['EarlyAllocator'].
    pub const fn new() -> Self {
        Self {
            start:0,
            end:0,
            b_pos:0,
            p_pos:0,
            byte_alloc_count:AtomicUsize::new(0),
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE>{
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = start + size;
        self.byte_alloc_count = AtomicUsize::new(0);
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        Err(AllocError::NoMemory)
    }
}


impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let align = layout.align();
        let size = layout.size();
        let aligned = (self.b_pos + align - 1) & !(align - 1);

        if aligned + size > self.p_pos {
            return Err(AllocError::NoMemory);
        }

        self.b_pos = aligned + size;
        self.byte_alloc_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        Ok(unsafe { NonNull::new_unchecked(aligned as *mut u8) })
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, layout: Layout) {
        let count = self.byte_alloc_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst);
        if count == 1 {
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.p_pos - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}


impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if !align_pow2.is_power_of_two() || align_pow2 < Self::PAGE_SIZE {
            return Err(AllocError::InvalidParam);
        }

        let size = num_pages * Self::PAGE_SIZE;
        let mut new_pos = self.p_pos - size;
        new_pos &= !(align_pow2 - 1);  // align downward

        if new_pos < self.b_pos {
            return Err(AllocError::NoMemory);
        }

        self.p_pos = new_pos;
        Ok(new_pos)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // Do nothing: no dealloc
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / Self::PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / Self::PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / Self::PAGE_SIZE
    }
}
