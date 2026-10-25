//! Bounded latest-value work slot for single-consumer native operations.
//! Replacing pending work drops obsolete source text immediately.
//! A running inference cannot be preempted; only pending work is coalesced.
use std::{
    sync::{Condvar, Mutex},
    time::Duration,
};

struct Inner<T> {
    pending: Option<T>,
    closed: bool,
}

pub enum Next<T> {
    Item(T),
    Timeout,
    Closed,
}

pub struct LatestSlot<T> {
    inner: Mutex<Inner<T>>,
    ready: Condvar,
}
impl<T> LatestSlot<T> {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner { pending: None, closed: false }),
            ready: Condvar::new(),
        }
    }

    /// Returns true when an earlier pending request was replaced.
    pub fn publish(&self, value: T) -> Result<bool, T> {
        let mut state = self.inner.lock().unwrap();
        if state.closed {
            return Err(value);
        }
        let replaced = state.pending.replace(value).is_some();
        // Wake worker without creating a thread per hotkey.
        self.ready.notify_one();
        Ok(replaced)
    }

    pub fn take_timeout(&self, timeout: Duration) -> Next<T> {
        let state = self.inner.lock().unwrap();
        let (mut state, _) = self.ready
            .wait_timeout_while(state, timeout, |slot| !slot.closed && slot.pending.is_none())
            .unwrap();
        if state.closed {
            return Next::Closed;
        }
        match state.pending.take() {
            Some(value) => Next::Item(value),
            None => Next::Timeout,
        }
    }

    pub fn close(&self) {
        let mut state = self.inner.lock().unwrap();
        state.closed = true;
        state.pending.take(); // promptly drop unprocessed sensitive input
        self.ready.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{mpsc, Arc};
    use std::thread;

    #[test]
    fn repeated_hotkeys_keep_only_last_pending_value() {
        let slot = LatestSlot::new();
        assert_eq!(slot.publish("obsolete-1").unwrap(), false);
        assert_eq!(slot.publish("obsolete-2").unwrap(), true);
        assert_eq!(slot.publish("newest").unwrap(), true);
        match slot.take_timeout(Duration::from_millis(0)) {
            Next::Item(value) => assert_eq!(value, "newest"),
            _ => panic!("latest work was lost"),
        };
        assert!(matches!(slot.take_timeout(Duration::from_millis(0)), Next::Timeout));
    }

    #[test]
    fn overlapping_producer_replaces_pending_work_during_inference() {
        let slot = Arc::new(LatestSlot::new());
        slot.publish(1usize).unwrap();
        let worker = slot.clone();
        let (running_tx, running_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let task = thread::spawn(move || {
            let first = match worker.take_timeout(Duration::from_secs(1)) {
                Next::Item(v) => v,
                _ => panic!("worker did not start"),
            };
            running_tx.send(first).unwrap();
            resume_rx.recv().unwrap(); // deterministic blocked inference boundary
            match worker.take_timeout(Duration::from_secs(1)) {
                Next::Item(v) => v,
                _ => panic!("pending latest item missing"),
            }
        });
        assert_eq!(running_rx.recv_timeout(Duration::from_secs(1)).unwrap(), 1);
        slot.publish(2).unwrap();
        slot.publish(3).unwrap();
        slot.publish(4).unwrap();
        resume_tx.send(()).unwrap();
        assert_eq!(task.join().unwrap(), 4);
    }

    #[test]
    fn close_discards_pending_input_and_wakes_consumer() {
        let slot = Arc::new(LatestSlot::<String>::new());
        let consumer = slot.clone();
        let (started_tx, started_rx) = mpsc::channel();
        let thread = thread::spawn(move || {
            started_tx.send(()).unwrap();
            matches!(consumer.take_timeout(Duration::from_secs(10)), Next::Closed)
        });
        started_rx.recv().unwrap();
        slot.close();
        assert!(thread.join().unwrap());
        assert_eq!(slot.publish("sensitive".to_owned()), Err("sensitive".to_owned()));
    }
}
