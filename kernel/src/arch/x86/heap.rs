pub fn init_heap() {
    unsafe extern "C" {
        static __heap_start: usize;
        static __heap_end: usize;
    }

    let heap_start = unsafe { __heap_start as *mut u8 };
    let heap_size = unsafe { __heap_end - __heap_start };
    unsafe { crate::kernel::HEAP_ALLOCATOR.lock().init(heap_start, heap_size); }
}
