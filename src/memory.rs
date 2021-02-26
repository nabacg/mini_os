use x86_64::{
    structures::paging::{FrameAllocator, PhysFrame, Mapper, OffsetPageTable, Page, PageTable, Size4KiB},
    PhysAddr, VirtAddr,
};

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3; // Cr3 points to level 4 page table

    let (level_4_table_frame, _) = Cr3::read(); // skipping flags on second field of tuple

    let phys = level_4_table_frame.start_address();
    // add physical_memory_offset we got from `bootloader` to calculate virtual memory where phys memory is 1-1 mapped to
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr(); // get a raw pointer to  PageTable out of the mem address

    &mut *page_table_ptr // as mutable since we'll need to modify PageTable
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000)); // VGA buffer address
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush(); // flush this page from TLB!

}


pub struct EmptyFrameAllocator; // allocator that always returns None

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}


use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

//frame allocator that returns usable frame using info from bootloader's memory_map
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {

    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get mem regions from memory_map
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        //map each region to it's address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // we pick one address every 4096 from the range, since we want 4Kb page frames
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create physframe
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
