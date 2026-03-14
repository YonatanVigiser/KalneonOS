use linked_list_allocator::LockedHeap;

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

unsafe extern "C" {
    static __heap_start: u8;
    static __heap_end: u8;
}

pub fn init() {
    let heap_start = unsafe { &__heap_start as *const u8 as *mut u8 };
    let heap_end = unsafe { &__heap_end as *const u8 };
    let heap_size = heap_end as usize - heap_start as usize;
    unsafe {
        HEAP_ALLOCATOR.lock().init(heap_start, heap_size);
    }
}
