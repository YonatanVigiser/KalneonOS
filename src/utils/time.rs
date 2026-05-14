use crate::platform::uptime_nano;

pub fn stall(nanos: u64) {
    let start = uptime_nano();
    while uptime_nano() < start + nanos {}
}
