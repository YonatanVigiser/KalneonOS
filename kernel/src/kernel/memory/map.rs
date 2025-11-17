use super::region::MemoryRegion;

pub struct MemoryMap {
    free_regions: Vec<MemoryRegion>,
}

impl MemoryMap {
    pub fn new(free_regions: &[MemoryRegion]) -> Self {
        let vec = Vec::new();
        vec.clone_from_slice(free_regions);
        vec.sort();
        Self {
            free_regions: vec,
        }
    }

    pub fn alloc(&mut self, size: usize) -> Option<MemoryRegion> {
        let size = size + (size % 
        let count = 0;
        for region in self.free_regions {
            if region.size() > size {
                return region.split(size);
            }
            else if region.size() == size {
                self.free_regions.remove(count);
                return Some(region);
            }
            count += 1;
        }
        None
    }

    pub fn free(&mut self, mut region: MemoryRegion) {
        let index = self.free_regions.iter().position(|test_region| region.start() >= test_region.end());
        let region_before = self.free_regions.get(index - 1);
        let region_after = self.free_regions.get(index + 1);
        if let Some(region_before) = region_before && let Some(merged_region) = region.join(region_before) {
            region = merged_region;
            self.free_regions.remove(index - 1)
        }

        if let Some(region_after) = self.free_regions.get(index - 1) && let Some(merged_region) = region.join(region_before) {
            region = merged_region;
        }
    }
}
