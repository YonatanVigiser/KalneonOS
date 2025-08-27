use super::cpu::{inb, io_wait, outb};
use core::sync::atomic::{AtomicU32, Ordering};

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;
const EOI: u8 = 0x20;

const ICW1_INIT: u8 = 0x10; /* Initialization - required! */
const ICW1_ICW4: u8 = 0x01; /* Indicates that ICW4 will be present */
/*
const ICW1_SINGLE: u8 =	0x02;		/* Single (cascade) mode */
const ICW1_INTERVAL4: u8 =	0x04;		/* Call address interval 4 (8) */
const ICW1_LEVEL: u8 = 0x08;		/* Level triggered (edge) mode */
*/

const ICW4_8086: u8 = 0x01; /* 8086/88 (MCS-80/85) mode */
/*
const ICW4_AUTO: u8 =	0x02;		/* Auto (normal) EOI */
const ICW4_BUF_SLAVE: u8 =	0x08;		/* Buffered mode/slave */
const ICW4_BUF_MASTER: u8 =	0x0C;		/* Buffered mode/master */
const ICW4_SFNM: u8 =0x10;		/* Special fully nested (not) */
*/

const CASCADE_IRQ: u8 = 2;

const PIC1_IDT_OFFSET: u8 = 0x20;
const PIC2_IDT_OFFSET: u8 = 0x28;

const PIC_READ_ISR: u8 = 0x0B;

static SPURIOUS_IRQS_COUNT: AtomicU32 = AtomicU32::new(0);

pub fn init() {
    // starts the initialization sequence (in cascade mode)
    outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();
    outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();
    outb(PIC1_DATA, PIC1_IDT_OFFSET);
    io_wait();
    outb(PIC2_DATA, PIC2_IDT_OFFSET);
    io_wait();
    // ICW3: tell Master PIC that there is a slave PIC at IRQ2
    outb(PIC1_DATA, 1 << CASCADE_IRQ);
    io_wait();
    // ICW3: tell Slave PIC its cascade identity (0000 0010)
    outb(PIC2_DATA, 2);
    io_wait();
    // ICW4: have the PICs use 8086 mode
    outb(PIC1_DATA, ICW4_8086);
    io_wait();
    outb(PIC2_DATA, ICW4_8086);
    io_wait();

    mask_all();
}

pub fn mask_irq(mut irq: u8) {
    let port: u16;
    if irq < 8 {
        port = PIC1_DATA;
    } else {
        port = PIC2_DATA;
        irq -= 8;
    }
    let value = inb(port) | (1 << irq);
    outb(port, value);
}

pub fn unmask_irq(mut irq: u8) {
    let port: u16;
    if irq < 8 {
        port = PIC1_DATA;
    } else {
        port = PIC2_DATA;
        irq -= 8;
    }
    let value = inb(port) & !(1 << irq);
    outb(port, value);
}

pub fn send_eoi(irq: u8) {
    if irq >= 8 {
        outb(PIC2_COMMAND, EOI);
    }
    outb(PIC1_COMMAND, EOI);
}

pub fn mask_all() {
    outb(PIC1_DATA, 0xff);
    outb(PIC2_DATA, 0xff);
}

pub fn unmask_all() {
    outb(PIC1_DATA, 0x00);
    outb(PIC2_DATA, 0x00);
}

pub fn read_isr() -> u16 {
    outb(PIC1_COMMAND, PIC_READ_ISR);
    outb(PIC2_COMMAND, PIC_READ_ISR);

    ((inb(PIC2_COMMAND) as u16) << 8) | inb(PIC1_COMMAND) as u16
}

pub fn spurios_irq(from_master: bool) {
    if !from_master {
        outb(PIC1_COMMAND, EOI);
    }
    SPURIOUS_IRQS_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn get_spurious_irqs_count() -> u32 {
    SPURIOUS_IRQS_COUNT.load(Ordering::Relaxed)
}

//pub fn set_clock_frq(
