use lazy_static::lazy_static;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::instructions::segmentation::*;
use x86_64::instructions::interrupts::without_interrupts;

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code_selector = gdt.append(Descriptor::kernel_code_segment());
        let kernel_data_selector = gdt.append(Descriptor::kernel_data_segment());
        (gdt, Selectors { kernel_code_selector, kernel_data_selector })
    };
}


struct Selectors {
    kernel_code_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
}

pub unsafe fn load() {
    without_interrupts(|| {
        GDT.0.load();
        unsafe {
            CS::set_reg(GDT.1.kernel_code_selector);
            SS::set_reg(GDT.1.kernel_data_selector);
            DS::set_reg(GDT.1.kernel_data_selector);
            ES::set_reg(GDT.1.kernel_data_selector);
            GS::set_reg(GDT.1.kernel_data_selector);
            FS::set_reg(GDT.1.kernel_data_selector);
        }
    });
}
