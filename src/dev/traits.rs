use crate::time::{KernelInstant, TimerResolution};

pub trait UptimeSource: Send + Sync {
    fn uptime(&self) -> KernelInstant;
    fn resolution(&self) -> TimerResolution;
}

pub trait LogSink: Send + Sync {
    fn log(&self, msg: &str);
}

pub trait CharOut: Send + Sync {
    fn out(&self, c: char);
}

pub trait GlobalIrqController: Send + Sync {
    fn set_irq_routing(&self, isa_irq: u8, dest_core_id: u8);
    fn set_masked(&self, isa_irq: u8, masked: bool);
}
