use crate::memory::HHDM_START;
use acpi::{AcpiTables, Handler, platform::AcpiPlatform};
use core::ptr::NonNull;
use x86_64::PhysAddr;
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};

pub fn platform_info(rsdt_addr: PhysAddr, rsdt_revision: u8) -> AcpiPlatform<AcpiRuntimeHandler> {
    log::info!("{:?}", rsdt_addr);
    let tables = unsafe {
        AcpiTables::from_rsdt(
            AcpiRuntimeHandler(),
            rsdt_revision,
            rsdt_addr.as_u64() as usize,
        )
    }
    .expect("Acpi tables parsing failed!");
    AcpiPlatform::new(tables, AcpiRuntimeHandler()).expect("Acpi platform creation failed!")
}

#[derive(Clone)]
pub struct AcpiRuntimeHandler();

impl Handler for AcpiRuntimeHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        let virtual_start =
            NonNull::new((physical_address + HHDM_START as usize) as *mut T).unwrap();
        acpi::PhysicalMapping {
            physical_start: physical_address,
            virtual_start,
            region_length: size,
            mapped_length: size,
            handler: self.clone(),
        }
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}

    fn read_u8(&self, address: usize) -> u8 {
        address as *const u8 as u8
    }

    fn read_u16(&self, address: usize) -> u16 {
        address as *const u16 as u16
    }

    fn read_u32(&self, address: usize) -> u32 {
        address as *const u32 as u32
    }

    fn read_u64(&self, address: usize) -> u64 {
        address as *const u64 as u64
    }

    fn write_u8(&self, address: usize, value: u8) {
        unsafe {
            (address as *mut u8).write(value);
        }
    }

    fn write_u16(&self, address: usize, value: u16) {
        unsafe {
            (address as *mut u16).write(value);
        }
    }

    fn write_u32(&self, address: usize, value: u32) {
        unsafe {
            (address as *mut u32).write(value);
        }
    }

    fn write_u64(&self, address: usize, value: u64) {
        unsafe {
            (address as *mut u64).write(value);
        }
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        unsafe { PortReadOnly::new(port).read() }
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        unsafe { PortReadOnly::new(port).read() }
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        unsafe { PortReadOnly::new(port).read() }
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        unsafe {
            PortWriteOnly::new(port).write(value);
        }
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        unsafe {
            PortWriteOnly::new(port).write(value);
        }
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        unsafe {
            PortWriteOnly::new(port).write(value);
        }
    }

    fn read_pci_u8(&self, _address: acpi::PciAddress, _offset: u16) -> u8 {
        todo!()
    }

    fn read_pci_u16(&self, _address: acpi::PciAddress, _offset: u16) -> u16 {
        todo!()
    }

    fn read_pci_u32(&self, _address: acpi::PciAddress, _offset: u16) -> u32 {
        todo!()
    }

    fn write_pci_u8(&self, _address: acpi::PciAddress, _offset: u16, _value: u8) {
        todo!();
    }

    fn write_pci_u16(&self, _address: acpi::PciAddress, _offset: u16, _value: u16) {
        todo!();
    }

    fn write_pci_u32(&self, _address: acpi::PciAddress, _offset: u16, _value: u32) {
        todo!();
    }

    fn nanos_since_boot(&self) -> u64 {
        crate::drivers::uptime_nano()
    }

    fn stall(&self, microseconds: u64) {
        crate::drivers::stall(microseconds * 1000)
    }

    fn sleep(&self, _milliseconds: u64) {
        todo!();
    }

    fn create_mutex(&self) -> acpi::Handle {
        todo!();
    }

    fn acquire(&self, _mutex: acpi::Handle, _timeout: u16) -> Result<(), acpi::aml::AmlError> {
        todo!();
    }

    fn release(&self, _mutex: acpi::Handle) {
        todo!();
    }
}
