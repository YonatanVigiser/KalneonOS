use core::pin::Pin;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::{Context, Poll};

use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use crossbeam_queue::ArrayQueue;
use futures_util::Stream;
use futures_util::task::AtomicWaker;
use pc_keyboard::{KeyCode, KeyState, Modifiers};
use spin::Mutex;

use crate::dev::registry::DEVICE_REGISTRY;
use crate::interrupt::apic::isa_irq_to_gsi;

pub mod ps2;

const PS2_KEYBOARD_ISA: u8 = 0x1;
const PS2_MOUSE_ISA: u8 = 0xC;

pub fn init() {
    DEVICE_REGISTRY.write().register::<dyn KeyboardEventIn>(Arc::new(KeyboardHub::default()));
    ps2::init(isa_irq_to_gsi(PS2_KEYBOARD_ISA), isa_irq_to_gsi(PS2_MOUSE_ISA));
}

#[derive(Debug, Clone)]
pub struct KeyEvent { pub keycode: KeyCode, pub keystate: KeyState, pub modifiers: Modifiers, pub unicode: Option<char> }

pub struct Subscriber {
    queue: ArrayQueue<KeyEvent>,
    waker: AtomicWaker,
    dropped: AtomicU32,
}

const KEY_EVENT_BUFFER_SIZE: usize = 32;

impl Subscriber {
    fn new() -> Self {
        Self {
            queue: ArrayQueue::new(KEY_EVENT_BUFFER_SIZE),
            waker: AtomicWaker::new(),
            dropped: AtomicU32::new(0),
        }
    }

    fn push(&self, event: KeyEvent) {
        if self.queue.push(event).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
        self.waker.wake();
    }

    pub fn take_dropped(&self) -> u32 {
        self.dropped.swap(0, Ordering::Relaxed)
    }
}

impl Subscriber {
    pub fn poll_next(&self, cx: &mut Context<'_>) -> Poll<KeyEvent> {
        if let Some(e) = self.queue.pop() { return Poll::Ready(e); }
        self.waker.register(cx.waker());
        match self.queue.pop() {
            Some(e) => { self.waker.take(); Poll::Ready(e) }
            None => Poll::Pending,
        }
    }
}

pub struct KeyboardReader(Arc<Subscriber>);

impl Stream for KeyboardReader {
    type Item = KeyEvent;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0.poll_next(cx).map(Some)
    }
}

pub trait KeyboardEventIn: Send + Sync {
    fn subscribe(&self) -> KeyboardReader;
    fn push(&self, event: KeyEvent);
}

#[derive(Default)]
pub struct KeyboardHub {
    subs: Mutex<Vec<Weak<Subscriber>>>,
}

impl KeyboardEventIn for KeyboardHub {
    fn subscribe(&self) -> KeyboardReader {
        let sub = Arc::new(Subscriber::new());
        self.subs.lock().push(Arc::downgrade(&sub));
        KeyboardReader(sub)
    }

    fn push(&self, event: KeyEvent) {
        let mut guard = self.subs.lock();
        guard.retain(|sub| {
            if let Some(sub) = sub.upgrade() {
                sub.push(event.clone());
                true
            } else { false }
        });
    }
}

