use crate::arch::x86::cpu::{inb, outb};

const COMMAND_PORT: u16 = 0x64;
const DATA_PORT: u16 = 0x60;
// Configuration byte for single port
const CONFIG: u8 = 0b00100100;
// Configuration byte for dual port
const DUAL_PORT_CONFIG: u8 = 0b00000100;

const SELF_TEST_SUCCESS: u8 = 0x55;

const TIMEOUT_COUNT: u32 = 10_000_000;

const DEVICE_RESET: u8 = 0xFF;
const DEVICE_RESEND_COMMAND: u8 = 0xFE;
const DEVICE_DISABLE_SCANNING: u8 = 0xF5;
const DEVICE_ENABLE_SCANNING: u8 = 0xF4;
const DEVICE_ACK: u8 = 0xFA;
const DEVICE_IDENTIFY: u8 = 0xF2;
const DEVICE_ECHO: u8 = 0xEE;

const DEVICE_MAX_RETRYS: u8 = 3;

use core::sync::atomic::{AtomicBool, Ordering};
pub static PORT1_SUPPORTED: AtomicBool = AtomicBool::new(false);
pub static PORT2_SUPPORTED: AtomicBool = AtomicBool::new(false);

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub enum PS2DeviceType {
    ATKeyboard,
    StandardMouse,
    ScrollwheelMouse,
    FiveButtonsMouse,
    MF2Keyboard,
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
    pub fn is_keyboard(&self) -> bool {
        match self {
            Self::ATKeyboard => true,
            Self::MF2Keyboard => true,
            Self::ShortKeyboard => true,
            Self::NCD97Keyboard => true,
            Self::Key122Keyboard => true,
            Self::JapaneseGKeyboard => true,
            Self::JapanesePKeyboard => true,
            Self::JapaneseAKeyboard => true,
            Self::NCDSunKeyboard => true,
            _ => false,
        }
    }

    pub fn is_mouse(&self) -> bool {
        !self.is_keyboard() && !matches!(self, Self::Unknown)
    }
}

impl PS2DeviceType {
    pub fn from(first_byte: Option<u8>, second_byte: Option<u8>) -> Self {
        match (first_byte, second_byte) {
            (None, None) => Self::ATKeyboard,
            (Some(0x00), _) => Self::StandardMouse,
            (Some(0x03), _) => Self::ScrollwheelMouse,
            (Some(0x04), _) => Self::FiveButtonsMouse,
            (Some(0xAB), Some(0x83) | Some(0xC1)) => Self::MF2Keyboard,
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
    // TODO: Add ps/2a controller support test (maybe before calling init)
    PORT1_SUPPORTED.store(false, Ordering::Release);
    PORT2_SUPPORTED.store(false, Ordering::Release);
    // Disable both ports
    send_command(PS2Command::DisableFirstPort);
    send_command(PS2Command::DisableSecondPort);
    // Drain stale data in data buffer
    drain_data_buffer();
    // Perform self test of the controller
    let config = read_config_byte();
    let config = config & 0b00100111;
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
    /* Uncomment when a proper mouse driver is implemented
    // Test if supported before testing status
    send_command(PS2Command::EnableSecondPort);
    if read_config_byte() & 0x20 == 0 {
        send_command(PS2Command::DisableSecondPort);
        reload_config(DUAL_PORT_CONFIG);
        port2_test = test_port2();
    } else {
        send_command(PS2Command::DisableSecondPort);
    }
    */
    let mut config = read_config_byte();
    if port1_test {
        send_command(PS2Command::EnableFirstPort);
        config |= 0x01;
        PORT1_SUPPORTED.store(true, Ordering::Release); // So reset wouldn't fail
        if let Err(_) = reset_device_port1() {
            PORT1_SUPPORTED.store(false, Ordering::Release);
            return Err(());
        }
    }
    if port2_test {
        send_command(PS2Command::EnableSecondPort);
        config |= 0x02;
        PORT2_SUPPORTED.store(true, Ordering::Release); // So reset wouldn't fail
        if let Err(_) = reset_device_port2() {
            PORT2_SUPPORTED.store(false, Ordering::Release);
            return Err(());
        }
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

/* Returns true if the data buffer isn't empty */
pub fn has_data() -> bool {
    (get_status_byte() & 0x01) != 0
}

/* Loops until can read from data port */
pub fn wait_for_read() {
    while !has_data() {}
}

/* Loops until can read from data port with a timeout.
 * If timeout passed return an error. */
pub fn wait_for_read_with_timeout(timeout: u32) -> Result<(), ()> {
    let mut count = 0;
    while !has_data() && count < timeout {
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
pub fn read_data_with_timeout() -> Result<u8, ()> {
    wait_for_read_with_timeout(TIMEOUT_COUNT)?;
    Ok(inb(DATA_PORT))
}

/* Clear the data input buffer */
pub fn drain_data_buffer() {
    while has_data() {
        let _ = inb(DATA_PORT);
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
pub fn disable_scanning_both_ports() {
    if PORT1_SUPPORTED.load(Ordering::Acquire) {
        let _ = send_device_data_port1(DEVICE_DISABLE_SCANNING);
        let _ = read_data_with_timeout();
    }
    if PORT2_SUPPORTED.load(Ordering::Acquire) {
        let _ = send_device_data_port2(DEVICE_DISABLE_SCANNING);
        let _ = read_data_with_timeout();
    }
}

/* Enable scanning on both supported ports */
pub fn enable_scanning_both_ports() {
    if PORT1_SUPPORTED.load(Ordering::Acquire) {
        let _ = send_device_data_port1(DEVICE_ENABLE_SCANNING);
        let _ = read_data_with_timeout();
    }
    if PORT2_SUPPORTED.load(Ordering::Acquire) {
        let _ = send_device_data_port2(DEVICE_ENABLE_SCANNING);
        let _ = read_data_with_timeout();
    }
}

/* Checks the connection to the device at port 1. Returns true if connected.
 * Note: IRQ must be masked for port 1, and must be unmasked for port 2 */
pub fn echo_device_port1() -> bool {
    // Send echo command
    if send_device_data_port1(DEVICE_ECHO).is_err() {
        return false;
    }

    // Check for echo response
    let result = if let Ok(response) = read_data_with_timeout()
        && response == DEVICE_ECHO
    {
        true
    } else {
        false
    };

    result
}

/* Checks the connection to the device at port 2. Returns true if connected.
 * Note: IRQ must be masked for port 2, and most be unmasked for port 1 */
pub fn echo_device_port2() -> bool {
    // Send echo command
    if send_device_data_port2(DEVICE_ECHO).is_err() {
        return false;
    }

    // Check for echo response
    let result = if let Ok(response) = read_data_with_timeout()
        && response == DEVICE_ECHO
    {
        true
    } else {
        false
    };

    result
}

/* Try sending data to device at port 1. Return error on timeout. */
pub fn send_device_data_port1(data: u8) -> Result<(), ()> {
    if !PORT1_SUPPORTED.load(Ordering::Acquire) {
        return Err(());
    }
    write_data_with_timeout(data, TIMEOUT_COUNT)
}

/* Try sending data to device at port 2. Return error on timeout. */
pub fn send_device_data_port2(data: u8) -> Result<(), ()> {
    if !PORT2_SUPPORTED.load(Ordering::Acquire) {
        return Err(());
    }
    send_command(PS2Command::SendDataToDevicePort2);
    write_data_with_timeout(data, TIMEOUT_COUNT)
}

/* Send command to a ps/2 device with optional data byte, with a wait for ACK from device */
pub fn send_command_device_port1(command: u8, data: Option<u8>) -> Result<(), ()> {
    let mut retrys = 0;
    while retrys < DEVICE_MAX_RETRYS {
        send_device_data_port1(command)?;
        if let Some(data) = data {
            send_device_data_port1(data)?;
        }
        let response = read_data_with_timeout()?;
        if response == DEVICE_ACK {
            return Ok(());
        } else if response == DEVICE_RESEND_COMMAND {
            retrys += 1;
        } else {
            return Err(())
        }
    }
    Err(())
}

/* Send command to a ps/2 device with optional data byte, with a wait for ACK from device */
pub fn send_command_device_port2(command: u8, data: Option<u8>) -> Result<(), ()> {
    let mut retrys = 0;
    while retrys < DEVICE_MAX_RETRYS {
        send_device_data_port2(command)?;
        if let Some(data) = data {
            send_device_data_port2(data)?;
        }
        let response = read_data_with_timeout()?;
        if response == DEVICE_ACK {
            return Ok(());
        } else if response == DEVICE_RESEND_COMMAND {
            retrys += 1;
        } else {
            return Err(())
        }
    }
    Err(())
}

/* Performs a device reset on port 1 */
pub fn reset_device_port1() -> Result<(), ()> {
    /* Disable scanning on both devices */
    disable_scanning_both_ports();

    // Drain any stale data
    drain_data_buffer();

    // Send reset command - renable scanning on failure
    if send_device_data_port1(DEVICE_RESET).is_err() {
        enable_scanning_both_ports();
        return Err(());
    }

    // Read response bytes
    let first_byte = read_data_with_timeout();
    let second_byte = read_data_with_timeout();

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

    // Drain any stale data
    drain_data_buffer();

    // Send reset command - renable scanning on failure
    if send_device_data_port2(DEVICE_RESET).is_err() {
        enable_scanning_both_ports();
        return Err(());
    }

    // Read response bytes
    let first_byte = read_data_with_timeout();
    let second_byte = read_data_with_timeout();

    // Re-enable scanning
    enable_scanning_both_ports();

    // Check if reset was successful
    match (first_byte, second_byte) {
        (Ok(0xFA), Ok(0xAA)) | (Ok(0xAA), Ok(0xFA)) => Ok(()),
        _ => Err(()),
    }
}

/* This is used to identify the device's types. Must be called when all supported ports are enabled */
pub fn identify_devices() -> (Option<PS2DeviceType>, Option<PS2DeviceType>) {
    // Disable scanning
    disable_scanning_both_ports();
    
    // Drain data buffer
    drain_data_buffer();

    let first_port_type = {
        if PORT1_SUPPORTED.load(Ordering::Acquire) && let Ok(_) = send_command_device_port1(DEVICE_IDENTIFY, None) {
            let first_byte = read_data_with_timeout().ok();
            let second_byte = read_data_with_timeout().ok();
            Some(PS2DeviceType::from(first_byte, second_byte))
        } else {
            None
        }
    };

    let second_port_type = {
        if PORT2_SUPPORTED.load(Ordering::Acquire) && let Ok(_) = send_command_device_port2(DEVICE_IDENTIFY, None) {
            let first_byte = read_data_with_timeout().ok();
            let second_byte = read_data_with_timeout().ok();
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
