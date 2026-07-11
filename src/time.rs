use acpi::HpetInfo;

pub mod hpet;
pub mod timer;

pub type KernelInstant = fugit::TimerInstant<u64, 1_000_000_000>;
pub type KernelDuration = fugit::NanosDurationU64;

pub fn init(hpet_info: HpetInfo) {
    hpet::HpetDriver::new(hpet_info);
}
