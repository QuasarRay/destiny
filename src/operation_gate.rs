//! Keep one public operation's ownership while its data lock is released for
//! a synchronous callback. Only the owning thread may reenter that operation.
use std::sync::{Condvar, Mutex};
use std::thread::{self, ThreadId};

#[derive(Default)]
struct State {
    owner: Option<ThreadId>,
    depth: usize,
}

#[derive(Default)]
pub(crate) struct OperationGate {
    state: Mutex<State>,
    available: Condvar,
}

pub(crate) struct OperationLease<'a> {
    gate: &'a OperationGate,
    _thread_bound: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl OperationGate {
    pub(crate) fn enter(&self) -> Result<OperationLease<'_>, &'static str> {
        let caller = thread::current().id();
        // Only bookkeeping is held here. No user/engine code runs under this
        // mutex, and unwind drops the lease. Engine poisoning is independent.
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        while state.owner.is_some_and(|owner| owner != caller) {
            state = self
                .available
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
        let next = state
            .depth
            .checked_add(1)
            .ok_or("operation reentry depth overflow")?;
        state.owner = Some(caller);
        state.depth = next;
        Ok(OperationLease {
            gate: self,
            _thread_bound: std::marker::PhantomData,
        })
    }
}

impl Drop for OperationLease<'_> {
    fn drop(&mut self) {
        let mut state = self
            .gate
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        // A lease never leaves its creating thread: ThreadId equality is the
        // concrete ownership relation. Do not expose leases to public callers.
        assert_eq!(state.owner, Some(thread::current().id()));
        assert!(state.depth > 0);
        state.depth -= 1;
        if state.depth == 0 {
            state.owner = None;
            self.gate.available.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier, mpsc};

    #[test]
    fn callback_reentry_keeps_other_owners_out_until_the_outer_lease_ends() {
        let gate = Arc::new(OperationGate::default());
        let outer = gate.enter().unwrap();
        let reentrant = gate.enter().unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let (started, start) = mpsc::channel();
        let (entered, entry) = mpsc::channel();
        let worker_gate = Arc::clone(&gate);
        let worker_barrier = Arc::clone(&barrier);
        let worker = thread::spawn(move || {
            started.send(()).unwrap();
            worker_barrier.wait();
            let _lease = worker_gate.enter().unwrap();
            entered.send(()).unwrap();
        });
        start.recv().unwrap();
        barrier.wait();
        drop(reentrant);
        assert_eq!(gate.state.lock().unwrap().depth, 1);
        assert!(entry.try_recv().is_err());
        drop(outer);
        entry.recv().unwrap();
        worker.join().unwrap();
    }

    #[test]
    fn unwind_releases_operation_ownership() {
        let gate = OperationGate::default();
        let result = std::panic::catch_unwind(|| {
            let _lease = gate.enter().unwrap();
            panic!("injected callback unwind");
        });
        assert!(result.is_err());
        let _lease = gate.enter().unwrap();
        assert_eq!(gate.state.lock().unwrap().depth, 1);
    }
}
