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

