use super::cpu::{inb, outb};

const COMMAND_PORT: u16 = 0x64;
const DATA_PORT: u16 = 0x60;
// Configuration byte for single port
const CONFIG: u8 = 0b00100100;
// Configuration byte for dual port
const DUAL_PORT_CONFIG: u8 = 0b00110100;

const SELF_TEST_SUCCESS: u8 = 0x55;

const TIMEOUT_COUNT: u32 = 1000;

const DEVICE_RESET: u8 = 0xFF;
const DEVICE_DISABLE_SCANNING: u8 = 0xF5;
const DEVICE_ENABLE_SCANNING: u8 = 0xF4;
const DEVICE_ACK: u8 = 0xFA;
const DEVICE_IDENTIFY: u8 = 0xF2;
const DEVICE_ECHO: u8 = 0xEE;

use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
pub static PORT1_SUPPORTED: AtomicBool = AtomicBool::new(false);
pub static PORT2_SUPPORTED: AtomicBool = AtomicBool::new(false);

#[repr(u8)]
pub enum PS2Command {
    ReadConfig = 0x20,
    ReloadConfig = 0x60,
    TestController = 0xAA,
    DisableSecondPort = 0xA7,
    EnableSecondPort = 0xA8,
    TestSecondPort = 0xA9,
    TestFirstPort = 0xAB,
    DisableFirstPort = 0xAD,
    EnableFirstPort = 0xAE,
    //ReadControllerOutputPort = 0xD0,
    SendDataToDevicePort2 = 0xD4,
    ResetCPU = 0xFE,
}

pub enum PS2DeviceType {
    ATKeyboard,
    StandardMouse,
    ScrollwheelMouse,
    FiveButtonsMouse,
    MF2Mouse,
    ShortKeyboard,
    NCD97Keyboard,
    Key122Keyboard,
    JapaneseGKeyboard,
    JapanesePKeyboard,
    JapaneseAKeyboard,
    NCDSunKeyboard,
    Unknown,
}

impl PS2DeviceType {
    pub fn from(first_byte: Option<u8>, second_byte: Option<u8>) -> Self {
        match (first_byte, second_byte) {
            (None, None) => Self::ATKeyboard,
            (Some(0x00), _) => Self::StandardMouse,
            (Some(0x03), _) => Self::ScrollwheelMouse,
            (Some(0x04), _) => Self::FiveButtonsMouse,
            (Some(0xAB), Some(0x83) | Some(0xC1)) => Self::MF2Mouse,
            (Some(0xAB), Some(0x84)) => Self::ShortKeyboard,
            (Some(0xAB), Some(0x85)) => Self::NCD97Keyboard,
            (Some(0xAB), Some(0x86)) => Self::Key122Keyboard,
            (Some(0xAB), Some(0x90)) => Self::JapaneseGKeyboard,
            (Some(0xAB), Some(0x91)) => Self::JapanesePKeyboard,
            (Some(0xAB), Some(0x92)) => Self::JapaneseAKeyboard,
            (Some(0xAC), Some(0xA1)) => Self::NCDSunKeyboard,
            _ => Self::Unknown,
        }
    }
}

/* Initialize the ps/2 controller and devices. Returns the two connected device types on success */
pub fn init() -> Result<(Option<PS2DeviceType>, Option<PS2DeviceType>), ()> {
    // TODO: Add ps/2 support detection test (maybe before calling init)
    PORT1_SUPPORTED.store(false, Relaxed);
    PORT2_SUPPORTED.store(false, Relaxed);
    // Disable both ports
    send_command(PS2Command::DisableFirstPort);
    send_command(PS2Command::DisableSecondPort);
    // Drain stale data in data buffer
    drain_data_buffer();
    // Perform self test of the controller
    let config = read_config_byte();
    let config = config & 0b1010110;
    reload_config(config);
    send_command(PS2Command::TestController);
    if read_data() != SELF_TEST_SUCCESS {
        return Err(());
    }
    // Reload config
    reload_config(CONFIG);
    // Test first port
    let port1_test = test_port1();
    // Test second port
    let mut port2_test = false;
    // Test if supported before testing status
    send_command(PS2Command::EnableSecondPort);
    if read_config_byte() & 0x20 == 0 {
        send_command(PS2Command::DisableSecondPort);
        reload_config(DUAL_PORT_CONFIG);
        port2_test = test_port2();
    } else {
        send_command(PS2Command::DisableSecondPort);
    }
    let mut config = read_config_byte();
    if port1_test {
        send_command(PS2Command::EnableFirstPort);
        config |= 0x01;
        PORT1_SUPPORTED.store(true, Relaxed);
        reset_device_port1()?;
    }
    if port2_test {
        send_command(PS2Command::EnableSecondPort);
        config |= 0x02;
        PORT2_SUPPORTED.store(true, Relaxed);
        reset_device_port2()?;
    }
    reload_config(config);
    // Identify connected devices
    Ok(identify_devices())
}

/* Loops until can write to data port or command port */
pub fn wait_for_write() {
    while (get_status_byte() & 0x02) != 0 {}
}

/* Loops until can write to data port or command port with a timeout.
 * If timeout passed return an error. */
pub fn wait_for_write_with_timeout(timeout: u32) -> Result<(), ()> {
    let mut count = 0;
    while (get_status_byte() & 0x02) != 0 && count < timeout {
        count += 1
    }
    if count < timeout { Ok(()) } else { Err(()) }
}

/* Loops until can read from data port */
pub fn wait_for_read() {
    while (get_status_byte() & 0x01) == 0 {}
}

/* Loops until can read from data port with a timeout.
 * If timeout passed return an error. */
pub fn wait_for_read_with_timeout(timeout: u32) -> Result<(), ()> {
    let mut count = 0;
    while (get_status_byte() & 0x01) == 0 && count < timeout {
        count += 1
    }
    if count < timeout { Ok(()) } else { Err(()) }
}

/* Get the ps/2 controller status byte */
pub fn get_status_byte() -> u8 {
    inb(COMMAND_PORT)
}

/* Send command to the ps/2 controller */
pub fn send_command(command: PS2Command) {
    wait_for_write();
    outb(COMMAND_PORT, command as u8);
}

/* Write to data port when possible */
pub fn write_data(data: u8) {
    wait_for_write();
    outb(DATA_PORT, data);
}

/* Write to data port with timeout. If timeout passed return error. */
pub fn write_data_with_timeout(data: u8, timeout: u32) -> Result<(), ()> {
    wait_for_write_with_timeout(timeout)?;
    outb(DATA_PORT, data);
    Ok(())
}

/* Read from data port when possible */
pub fn read_data() -> u8 {
    wait_for_read();
    inb(DATA_PORT)
}

/* Read from data port with timeout. If timeout passed return error. */
pub fn read_data_with_timeout(timeout: u32) -> Result<u8, ()> {
    wait_for_read_with_timeout(timeout)?;
    Ok(inb(DATA_PORT))
}

/* Clear the data input buffer with timeout protection */
pub fn drain_data_buffer() {
    let mut count = 0;
    while get_status_byte() & 0x01 != 0 && count < TIMEOUT_COUNT {
        let _ = inb(DATA_PORT);
        count += 1;
    }
}

/* Read the ps/2 controller config byte */
pub fn read_config_byte() -> u8 {
    send_command(PS2Command::ReadConfig);
    read_data()
}

/* Write config byte for the ps/2 controller */
pub fn reload_config(config: u8) {
    send_command(PS2Command::ReloadConfig);
    write_data(config);
}

/* Perform a test on port 1. Both ports MUST be disabled before calling this function.
 * Returning true if passed. */
pub fn test_port1() -> bool {
    send_command(PS2Command::TestFirstPort);
    read_data() == 0
}

/* Perform a test on port 2. Both ports MUST be disabled before calling this function.
 * Returning true if passed. */
pub fn test_port2() -> bool {
    send_command(PS2Command::TestSecondPort);
    read_data() == 0
}

/* Disable scanning on both supported ports */
fn disable_scanning_both_ports() {
    if PORT1_SUPPORTED.load(Relaxed) {
        let _ = send_device_data_port1(DEVICE_DISABLE_SCANNING);
        let _ = read_data_with_timeout(TIMEOUT_COUNT);
    }
    if PORT2_SUPPORTED.load(Relaxed) {
        let _ = send_device_data_port2(DEVICE_DISABLE_SCANNING);
        let _ = read_data_with_timeout(TIMEOUT_COUNT);
    }
}

/* Enable scanning on both supported ports */
fn enable_scanning_both_ports() {
    if PORT1_SUPPORTED.load(Relaxed) {
        let _ = send_device_data_port1(DEVICE_ENABLE_SCANNING);
        let _ = read_data_with_timeout(TIMEOUT_COUNT);
    }
    if PORT2_SUPPORTED.load(Relaxed) {
        let _ = send_device_data_port2(DEVICE_ENABLE_SCANNING);
        let _ = read_data_with_timeout(TIMEOUT_COUNT);
    }
}

/* Checks the connection to the device at port 1. Returns true if connected.
 * Note: This will drain any stale data in the data buffer. */
pub fn echo_device_port1() -> bool {
    if !PORT1_SUPPORTED.load(Relaxed) {
        return false;
    }

    // Disable scanning on both devices to prevent interference
    disable_scanning_both_ports();

    // Drain any stale data
    drain_data_buffer();

    // Send echo command
    if send_device_data_port1(DEVICE_ECHO).is_err() {
        enable_scanning_both_ports();
        return false;
    }

    // Check for echo response
    let result = if let Ok(response) = read_data_with_timeout(TIMEOUT_COUNT)
        && response == DEVICE_ECHO
    {
        true
    } else {
        false
    };

    // Re-enable scanning on both devices
    enable_scanning_both_ports();

    result
}

/* Checks the connection to the device at port 2. Returns true if connected.
 * Note: This will drain any stale data in the data buffer. */
pub fn echo_device_port2() -> bool {
    if !PORT2_SUPPORTED.load(Relaxed) {
        return false;
    }

    // Disable scanning on both devices to prevent interference
    disable_scanning_both_ports();

    // Drain any stale data
    drain_data_buffer();

    // Send echo command
    if send_device_data_port2(DEVICE_ECHO).is_err() {
        enable_scanning_both_ports();
        return false;
    }

    // Check for echo response
    let result = if let Ok(response) = read_data_with_timeout(TIMEOUT_COUNT)
        && response == DEVICE_ECHO
    {
        true
    } else {
        false
    };

    // Re-enable scanning on both devices
    enable_scanning_both_ports();

    result
}

/* Try sending data to device at port 1. Return error on timeout. */
pub fn send_device_data_port1(data: u8) -> Result<(), ()> {
    if !PORT1_SUPPORTED.load(Relaxed) {
        return Err(());
    }
    write_data_with_timeout(data, TIMEOUT_COUNT)
}

/* Try sending data to device at port 2. Return error on timeout. */
pub fn send_device_data_port2(data: u8) -> Result<(), ()> {
    if !PORT2_SUPPORTED.load(Relaxed) {
        return Err(());
    }
    send_command(PS2Command::SendDataToDevicePort2);
    write_data_with_timeout(data, TIMEOUT_COUNT)
}

/* Performs a device reset on port 1 */
pub fn reset_device_port1() -> Result<(), ()> {
    /* Disable scanning on both devices */
    disable_scanning_both_ports();

    // Send reset command - renable scanning on failure
    if send_device_data_port1(DEVICE_RESET).is_err() {
        enable_scanning_both_ports();
        return Err(());
    }

    // Drain any stale data
    drain_data_buffer();

    // Read response bytes
    let first_byte = read_data_with_timeout(TIMEOUT_COUNT);
    let second_byte = read_data_with_timeout(TIMEOUT_COUNT);

    // Re-enable scanning
    enable_scanning_both_ports();

    // Check if reset was successful
    match (first_byte, second_byte) {
        (Ok(0xFA), Ok(0xAA)) | (Ok(0xAA), Ok(0xFA)) => Ok(()),
        _ => Err(()),
    }
}

/* Performs a device reset on port 2 */
pub fn reset_device_port2() -> Result<(), ()> {
    /* Disable scanning on both devices */
    disable_scanning_both_ports();

    // Send reset command - renable scanning on failure
    if send_device_data_port1(DEVICE_RESET).is_err() {
        enable_scanning_both_ports();
        return Err(());
    }

    // Drain any stale data
    drain_data_buffer();

    // Read response bytes
    let first_byte = read_data_with_timeout(TIMEOUT_COUNT);
    let second_byte = read_data_with_timeout(TIMEOUT_COUNT);

    // Re-enable scanning
    enable_scanning_both_ports();

    // Check if reset was successful
    match (first_byte, second_byte) {
        (Ok(0xFA), Ok(0xAA)) | (Ok(0xAA), Ok(0xFA)) => Ok(()),
        _ => Err(()),
    }
}

/* This is used to identify the device's types. Must be called when both ports are enabled */
pub fn identify_devices() -> (Option<PS2DeviceType>, Option<PS2DeviceType>) {
    // Disable scanning
    disable_scanning_both_ports();

    let first_port_type = {
        if PORT1_SUPPORTED.load(Relaxed) && let Ok(_) = send_device_data_port1(DEVICE_IDENTIFY) && let Ok(response) = read_data_with_timeout(TIMEOUT_COUNT) && response == DEVICE_ACK {
            let first_byte = read_data_with_timeout(TIMEOUT_COUNT).ok();
            let second_byte = read_data_with_timeout(TIMEOUT_COUNT).ok();
            Some(PS2DeviceType::from(first_byte, second_byte))
        } else {
            None
        }
    };

    let second_port_type = {
        if PORT2_SUPPORTED.load(Relaxed) && let Ok(_) = send_device_data_port2(DEVICE_IDENTIFY) && let Ok(response) = read_data_with_timeout(TIMEOUT_COUNT) && response == DEVICE_ACK {
            let first_byte = read_data_with_timeout(TIMEOUT_COUNT).ok();
            let second_byte = read_data_with_timeout(TIMEOUT_COUNT).ok();
            Some(PS2DeviceType::from(first_byte, second_byte))
        } else {
            None
        }
    };

    // Re-enable scanning
    enable_scanning_both_ports();

    (first_port_type, second_port_type)
}

/* Send a CPU reset signal. Should reset the CPU */
pub unsafe fn reset_cpu() -> ! {
    send_command(PS2Command::ResetCPU);
    panic!("CPU reset didn't work as expected. Panic instead")
}

/* Exploit a bug in the way that the CPU reset signal is sent so that the CPU
 * enters an unrecoverable reseting loop - essentially blocking the CPU from starting
 * until RAM resets (via power loss). NOT RECOMMENDED - but very funny */
pub unsafe fn kill_cpu() -> ! {
    reload_config(get_status_byte() | 0x01);
    panic!("CPU kill didn't work as expected. Panic instead")
}
