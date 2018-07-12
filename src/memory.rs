pub use arch::paging::*;
use bit_allocator::{BitAlloc, BitAlloc64K};
use consts::MEMORY_OFFSET;
use spin::{Mutex, MutexGuard};
use super::HEAP_ALLOCATOR;
use ucore_memory::{*, paging::PageTable};
#[cfg(target_arch = "x86_64")]
use ucore_memory::cow::CowExt;
pub use ucore_memory::memory_set::{MemoryArea, MemoryAttr, MemorySet as MemorySet_, Stack};

pub type MemorySet = MemorySet_<InactivePageTable0>;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<BitAlloc64K> = Mutex::new(BitAlloc64K::default());
}

pub fn alloc_frame() -> Option<usize> {
    FRAME_ALLOCATOR.lock().alloc().map(|id| id * PAGE_SIZE + MEMORY_OFFSET)
}

pub fn dealloc_frame(target: usize) {
    FRAME_ALLOCATOR.lock().dealloc((target - MEMORY_OFFSET) / PAGE_SIZE);
}

// alloc from heap
pub fn alloc_stack() -> Stack {
    use alloc::boxed::Box;
    const STACK_SIZE: usize = 0x8000;
    #[repr(align(0x8000))]
    struct StackData([u8; STACK_SIZE]);
    let data = Box::new(StackData([0; STACK_SIZE]));
    let bottom = Box::into_raw(data) as usize;
    let top = bottom + STACK_SIZE;
    Stack { top, bottom }
}

#[cfg(target_arch = "x86_64")]
lazy_static! {
    static ref ACTIVE_TABLE: Mutex<CowExt<ActivePageTable>> = Mutex::new(unsafe {
        CowExt::new(ActivePageTable::new())
    });
}

#[cfg(target_arch = "riscv")]
lazy_static! {
    static ref ACTIVE_TABLE: Mutex<ActivePageTable> = Mutex::new(unsafe {
        ActivePageTable::new()
    });
}

/// The only way to get active page table
#[cfg(target_arch = "x86_64")]
pub fn active_table() -> MutexGuard<'static, CowExt<ActivePageTable>> {
    ACTIVE_TABLE.lock()
}

#[cfg(target_arch = "riscv")]
pub fn active_table() -> MutexGuard<'static, ActivePageTable> {
    ACTIVE_TABLE.lock()
}

// Return true to continue, false to halt
#[cfg(target_arch = "x86_64")]
pub fn page_fault_handler(addr: usize) -> bool {
    // Handle copy on write
    unsafe { ACTIVE_TABLE.force_unlock(); }
    active_table().page_fault_handler(addr, || alloc_frame().unwrap())
}

#[cfg(target_arch = "riscv")]
pub fn page_fault_handler(addr: usize) -> bool {
    false
}

pub fn init_heap() {
    use consts::{KERNEL_HEAP_OFFSET, KERNEL_HEAP_SIZE};
    unsafe { HEAP_ALLOCATOR.lock().init(KERNEL_HEAP_OFFSET, KERNEL_HEAP_SIZE); }
    info!("heap init end");
}

//pub mod test {
//    pub fn cow() {
//        use super::*;
//        use ucore_memory::cow::test::test_with;
//        test_with(&mut active_table());
//    }
//}