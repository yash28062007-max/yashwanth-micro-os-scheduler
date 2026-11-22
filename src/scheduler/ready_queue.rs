//! Ready Queue Data Structure
//!
//! Implements a bounded, deterministic FIFO ring buffer queue for runnable tasks.
//! Ensures safe, bounded-time O(1) enqueue and dequeue operations without heap allocation.

use crate::task::TaskId;

pub const MAX_TASKS: usize = 16;

/// Bounded FIFO Ready Queue for kernel task scheduling.
pub struct ReadyQueue {
    buffer: [Option<TaskId>; MAX_TASKS],
    head: usize,
    tail: usize,
    count: usize,
}

impl ReadyQueue {
    /// Creates a new empty ready queue.
    pub const fn new() -> Self {
        Self {
            buffer: [None; MAX_TASKS],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    /// Enqueues a task ID to the tail of the ready queue.
    pub fn enqueue(&mut self, id: TaskId) -> Result<(), ()> {
        if self.count >= MAX_TASKS {
            return Err(()); // Queue full
        }
        self.buffer[self.tail] = Some(id);
        self.tail = (self.tail + 1) % MAX_TASKS;
        self.count += 1;
        Ok(())
    }

    /// Dequeues the next runnable task ID from the head of the ready queue.
    pub fn dequeue(&mut self) -> Option<TaskId> {
        if self.count == 0 {
            return None;
        }
        let id = self.buffer[self.head].take();
        self.head = (self.head + 1) % MAX_TASKS;
        self.count -= 1;
        id
    }

    /// Peeks at the next task ID without removing it.
    pub fn peek(&self) -> Option<TaskId> {
        if self.count == 0 {
            None
        } else {
            self.buffer[self.head]
        }
    }

    /// Returns the number of tasks in the ready queue.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Checks if the ready queue is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Checks if the ready queue is full.
    pub fn is_full(&self) -> bool {
        self.count >= MAX_TASKS
    }

    /// Checks if a task is already queued.
    pub fn contains(&self, id: TaskId) -> bool {
        for i in 0..self.count {
            let idx = (self.head + i) % MAX_TASKS;
            if self.buffer[idx] == Some(id) {
                return true;
            }
        }
        false
    }

    /// Removes a specific task ID if present in the queue.
    pub fn remove(&mut self, id: TaskId) -> bool {
        if self.count == 0 {
            return false;
        }

        let mut found = false;
        let mut temp_tasks = [None; MAX_TASKS];
        let mut temp_count = 0;

        while let Some(task) = self.dequeue() {
            if task == id && !found {
                found = true;
            } else {
                temp_tasks[temp_count] = Some(task);
                temp_count += 1;
            }
        }

        for i in 0..temp_count {
            if let Some(t) = temp_tasks[i] {
                let _ = self.enqueue(t);
            }
        }

        found
    }
}
