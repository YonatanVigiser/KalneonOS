use multiboot2::{BootInformation, BootInformationHeader};

pub struct BootInfo {
}

pub fn init(boot_magic: u32, boot_info_ptr: u32) -> BootInfo {
    if boot_magic == multiboot2::MAGIC {
        let boot_info = unsafe { BootInformation::load(boot_info_ptr as *const BootInformationHeader).unwrap() };
    }
    panic!("No multiboot magic found!");
}
