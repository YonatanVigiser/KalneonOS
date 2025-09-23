use crate::boot::boot_info;

#[repr(u32)]
#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum MemoryType {
    Usable = 1,
    Reserved = 2,
    APICReclaimable = 3,
    APICNVS = 4,
    Bad = 5,
}

impl MemoryType {
    pub fn is_usable(&self) -> bool {
        if let MemoryType::Usable | MemoryType::APICReclaimable = self {
            return true;
        }
        false
    }
}

impl TryFrom<u32> for MemoryType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Usable),
            2 => Ok(Self::Reserved),
            3 => Ok(Self::APICReclaimable),
            4 => Ok(Self::APICNVS),
            5 => Ok(Self::Bad),
            _ => Err(())
        }
    }
}


#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct MemoryMapEntry {
    pub start: usize,
    pub end: usize,
    pub mem_type: MemoryType,
}

const MMAP_SIZE: usize = boot_info::MMAP_SIZE;

pub struct MemoryMap([MemoryMapEntry; MMAP_SIZE]);

impl MemoryMap {
    pub fn from_bios_mmap(bios_mmap: &[boot_info::MemoryMapEntry]) -> Self {
        let mut mmap = Self([MemoryMapEntry { start: usize::MAX, end: usize::MAX, mem_type: MemoryType::Reserved }; MMAP_SIZE]);
        let mut filled_length = 0;
        for (count, bios_entry) in bios_mmap.iter().enumerate() {
            if bios_entry.length == 0 || bios_entry.mem_type == 0 {
                continue;
            }
            let entry = &mut mmap.0[count];
            entry.start = bios_entry.base as usize;
            entry.end = bios_entry.base as usize + bios_entry.length as usize;
            entry.mem_type = bios_entry.mem_type.try_into().unwrap_or(MemoryType::Reserved);
            filled_length += 1;
        }
        mmap.0.sort_unstable();
        let mut last_end = 0;
        for entry in mmap.0 {
            if entry.start > last_end {
                mmap.0[filled_length] = MemoryMapEntry {
                    start: last_end,
                    end: entry.start,
                    mem_type: MemoryType::Reserved
                };
                filled_length += 1;
            }
            last_end = entry.start;
        }

        mmap
    }

    pub fn size() -> usize {
        MMAP_SIZE
    }
}
