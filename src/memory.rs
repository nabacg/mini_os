use x86_64::PhysAddr;
use x86_64::{structures::paging::PageTable, VirtAddr};

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3; // Cr3 points to level 4 page table

    let (level_4_table_frame, _) = Cr3::read(); // skipping flags on second field of tuple

    let phys = level_4_table_frame.start_address();
    // add physical_memory_offset we got from `bootloader` to calculate virtual memory where phys memory is 1-1 mapped to
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr(); // get a raw pointer to  PageTable out of the mem address

    &mut *page_table_ptr // as mutable since we'll need to modify PageTable
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    let (level_4_table_frame, _) = Cr3::read(); // read active level4 table from CR3 register
                                                // extract indexes for 4 levels of PT from the addr (access different bits of the address)
    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];

    let mut frame = level_4_table_frame; // starting at level4 frame

    // iterate over levels
    for &index in &table_indexes {
        // convert physical address to virtual, by adding physical_memory_offset
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let pt_ptr: *const PageTable = virt.as_ptr();
        let page_table = unsafe { &*pt_ptr };

        //get the entry for current level index
        let entry = &page_table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Huge Pages not supported!"),
        }
    }

    //finally here calculate the actual address using physical frame address and virtual address offset
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`.
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    // using inner function to limit the scope of `unsafe fn`
    translate_addr_inner(addr, physical_memory_offset)
}

use x86_64::structures::paging::OffsetPageTable;

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}
