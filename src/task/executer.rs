use core::task::Waker;

use alloc::{collections::btree_map::BTreeMap, sync::Weak};

use crate::task::{Task, TaskId};

pub struct Executor {
    tasks: BTreeMap<TaskId, Weak<Task>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub const fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
    }
}
