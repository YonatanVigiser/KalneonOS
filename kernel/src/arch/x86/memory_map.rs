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

const MMAP_SIZE: usize = 256;

pub struct MemoryMap([MemoryMapEntry; MMAP_SIZE]);

impl MemoryMap {
    pub fn from_bios_mmap(bios_mmap: &[boot_info::MemoryMapEntry]) -> Self {
        let mut mmap = Self([MemoryMapEntry { start: 0, end: 0, mem_type: MemoryType::Reserved }; MMAP_SIZE]);
        for (count, entry) in bios_mmap.iter().enumerate() {
            mmap.0[count].start = entry.base as usize;
            mmap.0[count].end = entry.base as usize + entry.length as usize;
            mmap.0[count].mem_type = entry.mem_type.try_into().unwrap_or(MemoryType::Reserved);
        }
        mmap.0.sort_unstable();
        let mut holes_counter = 0;
        for entry_index in 1..bios_mmap.len() {
            if mmap.0[entry_index - 1].end < mmap.0[entry_index].start {
                mmap.0[bios_mmap.len() + holes_counter] = MemoryMapEntry {
                    start: mmap.0[entry_index - 1].end,
                    end: mmap.0[entry_index].start,
                    mem_type: MemoryType::Reserved,
                };
                holes_counter += 1;
            }
            if mmap.0[entry_index - 1].end > mmap.0[entry_index].start {
                if mmap.0[entry_index - 1].mem_type.is_usable() {
                } else {
                }
            }
        }
        mmap
    }

    pub fn size() -> usize {
        MMAP_SIZE
    }
}
