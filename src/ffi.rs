use std::{
    any::Any,
    collections::VecDeque,
    io::{self, Write},
    panic::{AssertUnwindSafe, catch_unwind},
    ptr, slice,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use serde_json::{Value, json};

use crate::ffi_lifecycle::{release_attempt, retain_attempt};
use crate::operation_gate::OperationGate;
use crate::runtime::{
    CallCallback, CompatOptions, CompatRequest, CompatRuntime, strict_json_from_slice,
};

pub const DBC_ABI_VERSION: u32 = 1;
const MAX_REQUEST_BYTES: usize = 48 * 1024 * 1024;
const MAX_RESPONSE_BYTES: usize = 48 * 1024 * 1024;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DbcBuffer {
    pub data: *mut u8,
    pub len: usize,
    pub status: i32,
}

impl DbcBuffer {
    const fn empty(status: i32) -> Self {
        Self {
            data: ptr::null_mut(),
            len: 0,
            status,
        }
    }
}

#[repr(C)]
pub struct DbcRuntime {
    runtime: Mutex<CompatRuntime>,
    poisoned: AtomicBool,
    references: AtomicUsize,
    operations: OperationGate,
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_abi_version() -> u32 {
    DBC_ABI_VERSION
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_runtime_new(
    options_json: *const u8,
    options_len: usize,
    out_error: *mut DbcBuffer,
) -> *mut DbcRuntime {
    set_out_error(out_error, DbcBuffer::empty(0));
    let result = catch_unwind(AssertUnwindSafe(|| {
        let bytes = input_bytes(options_json, options_len)?;
        let options = if bytes.is_empty() {
            CompatOptions::default()
        } else {
            strict_json_from_slice(&bytes)
                .map_err(|error| format!("invalid options JSON: {error}"))?
        };
        let runtime = CompatRuntime::new(options).map_err(|error| error.to_string())?;
        Ok::<_, String>(Box::into_raw(Box::new(DbcRuntime {
            runtime: Mutex::new(runtime),
            poisoned: AtomicBool::new(false),
            references: AtomicUsize::new(1),
            operations: OperationGate::default(),
        })))
    }));
    match result {
        Ok(Ok(runtime)) => runtime,
        Ok(Err(message)) => {
            set_out_error(out_error, diagnostic_buffer(message, 1));
            ptr::null_mut()
        }
        Err(payload) => {
            set_out_error(out_error, diagnostic_buffer(panic_message(payload), 2));
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_runtime_free(runtime: *mut DbcRuntime) {
    dbc_runtime_release(runtime);
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_runtime_retain(runtime: *mut DbcRuntime) -> bool {
    if runtime.is_null() {
        return false;
    }
    // SAFETY: the caller must already own a live reference. The increment is
    // what permits that reference to be shared with another owner.
    let runtime = unsafe { &*runtime };
    // This is the explicit weak-CAS retry form of fetch_update. The actual
    // attempt body is shared with Verus. A failed attempt changes no count;
    // retry progress still depends on atomic/scheduler fairness.
    let mut expected = runtime.references.load(Ordering::SeqCst);
    loop {
        match retain_attempt(&runtime.references, expected) {
            None => return false,
            Some(Ok(_)) => return true,
            Some(Err(actual)) => expected = actual,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_runtime_release(runtime: *mut DbcRuntime) {
    if runtime.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: the caller transfers one live reference to this function.
        let runtime_ref = unsafe { &*runtime };
        let mut expected = runtime_ref.references.load(Ordering::SeqCst);
        let previous = loop {
            match release_attempt(&runtime_ref.references, expected) {
                None => break None,
                Some(Ok(previous)) => break Some(previous),
                Some(Err(actual)) => expected = actual,
            }
        };
        if previous == Some(1) {
            // The successful SeqCst RMW is the final-reference transition.
            // It includes acquire ordering before reclaiming the allocation.
            unsafe {
                drop(Box::from_raw(runtime));
            }
        }
    }));
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_runtime_call(
    runtime: *mut DbcRuntime,
    request_json: *const u8,
    request_len: usize,
) -> DbcBuffer {
    dbc_runtime_call_with_mode_callback(runtime, request_json, request_len, None, ptr::null_mut())
}

pub type DbcModeCallback = extern "C" fn(*mut std::ffi::c_void, i64, u8, u8);
pub type DbcEventCallback = extern "C" fn(*mut std::ffi::c_void, i64, u32, *const u8, usize);

/// Additive ABI entry. A callback must return synchronously without unwinding.
/// The data mutex is released for reentry; public-operation ownership remains
/// held so another thread cannot observe a partly completed tick.
#[unsafe(no_mangle)]
pub extern "C" fn dbc_runtime_call_with_mode_callback(
    runtime: *mut DbcRuntime,
    request_json: *const u8,
    request_len: usize,
    callback: Option<DbcModeCallback>,
    context: *mut std::ffi::c_void,
) -> DbcBuffer {
    protected_runtime_operation(runtime, || {
        runtime_call_inner(runtime, request_json, request_len, callback, None, context)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_runtime_call_with_event_callback(
    runtime: *mut DbcRuntime,
    request_json: *const u8,
    request_len: usize,
    callback: Option<DbcEventCallback>,
    context: *mut std::ffi::c_void,
) -> DbcBuffer {
    protected_runtime_operation(runtime, || {
        runtime_call_inner(runtime, request_json, request_len, None, callback, context)
    })
}

struct RuntimeCallLease(*mut DbcRuntime);
impl RuntimeCallLease {
    fn new(runtime: *mut DbcRuntime) -> Option<Self> {
        if dbc_runtime_retain(runtime) {
            Some(Self(runtime))
        } else {
            None
        }
    }
}

fn protected_runtime_operation(
    runtime: *mut DbcRuntime,
    operation: impl FnOnce() -> DbcBuffer,
) -> DbcBuffer {
    if runtime.is_null() {
        return response_buffer(
            json!({"ok":false,"error":{"code":"invalid_runtime","message":"runtime is null"}}),
            1,
        );
    }
    let Some(_call_reference) = RuntimeCallLease::new(runtime) else {
        return response_buffer(
            json!({"ok":false,"error":{"code":"reference_limit","message":"could not retain the synchronous call reference"}}),
            1,
        );
    };
    // SAFETY: the caller provided a live reference and the call lease retains
    // it even if a reentrant callback releases the caller's original owner.
    let runtime_ref = unsafe { &*runtime };
    let _operation = match runtime_ref.operations.enter() {
        Ok(lease) => lease,
        Err(message) => {
            return response_buffer(
                json!({"ok":false,"error":{"code":"reentry_limit","message":message}}),
                1,
            );
        }
    };
    // Ownership and the call reference outlive panic reporting. Another owner
    // cannot enter between an unwind and publication of the poison flag.
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(buffer) => buffer,
        Err(payload) => {
            runtime_ref.poisoned.store(true, Ordering::Release);
            response_buffer(
                json!({"ok":false,"error":{"code":"panic","message":panic_message(payload)}}),
                2,
            )
        }
    }
}
impl Drop for RuntimeCallLease {
    fn drop(&mut self) {
        dbc_runtime_release(self.0);
    }
}

fn drain_callbacks(
    runtime: &DbcRuntime,
    modes: &mut VecDeque<CallCallback>,
    proximities: &mut VecDeque<Value>,
    callback: Option<DbcModeCallback>,
    event_callback: Option<DbcEventCallback>,
    context: *mut std::ffi::c_void,
) -> Result<(), String> {
    loop {
        let (event, entered) = {
            let mut guard = runtime
                .runtime
                .lock()
                .map_err(|_| "runtime data lock is poisoned")?;
            let event = modes.pop_front();
            if event
                .as_ref()
                .is_some_and(|event| !guard.callback_is_live(event))
            {
                continue;
            }
            if matches!(
                event,
                Some(
                    CallCallback::Remove { .. }
                        | CallCallback::GotoPoint { .. }
                        | CallCallback::Follow { .. }
                        | CallCallback::RegisterFollower { .. }
                        | CallCallback::WarpStart { .. }
                        | CallCallback::EntityArrival { .. }
                        | CallCallback::SetMode { .. }
                        | CallCallback::FormationChange { .. }
                )
            ) {
                guard
                    .complete_mode_callback(event.expect("internal continuation"))
                    .map_err(|error| error.to_string())?;
                guard.prepend_callback_continuations(modes);
                continue;
            }
            let proximity = if event.is_none() {
                loop {
                    let Some(candidate) = proximities.pop_front() else {
                        break None;
                    };
                    if guard.proximity_callback_is_live(&candidate) {
                        break Some(candidate);
                    }
                }
            } else {
                None
            };
            if event.is_none() && proximity.is_none() {
                return Ok(());
            }
            let entered = guard.enter_mode_callback();
            ((event, proximity), entered)
        };
        // No &mut CompatRuntime or MutexGuard crosses this foreign call.
        if let Some(CallCallback::Mode(mode)) = event.0.as_ref() {
            if let Some(observer) = event_callback {
                let args = serde_json::to_vec(&json!([mode.old_mode, mode.new_mode]))
                    .map_err(|error| error.to_string())?;
                observer(context, mode.ball_id, 8, args.as_ptr(), args.len());
            } else if let Some(observer) = callback {
                observer(context, mode.ball_id, mode.old_mode, mode.new_mode);
            }
        } else if let Some(CallCallback::Park { kind, arguments }) = event.0.as_ref()
            && let Some(observer) = event_callback
        {
            let args = serde_json::to_vec(arguments).map_err(|error| error.to_string())?;
            observer(context, 0, *kind, args.as_ptr(), args.len());
        } else if let Some(CallCallback::Ball {
            ball_id,
            kind,
            arguments,
            ..
        }) = event.0.as_ref()
            && let Some(observer) = event_callback
        {
            let args = serde_json::to_vec(arguments).map_err(|error| error.to_string())?;
            observer(context, *ball_id, *kind, args.as_ptr(), args.len());
        } else if let Some(proximity) = event.1.as_ref()
            && let Some(observer) = event_callback
        {
            let owner = proximity["owner_id"]
                .as_i64()
                .ok_or("invalid proximity owner")?;
            let kind = proximity
                .get("event_kind")
                .and_then(Value::as_u64)
                .unwrap_or(3) as u32;
            let arguments = proximity.get("arguments").cloned().unwrap_or_else(|| {
                json!([
                    proximity["other_id"],
                    if proximity["entering"] == json!(true) {
                        1
                    } else {
                        0
                    }
                ])
            });
            let args = serde_json::to_vec(&arguments).map_err(|error| error.to_string())?;
            observer(context, owner, kind, args.as_ptr(), args.len());
        }
        let mut guard = runtime
            .runtime
            .lock()
            .map_err(|_| "runtime data lock is poisoned")?;
        if let Some(mode) = event.0 {
            guard
                .complete_mode_callback(mode)
                .map_err(|error| error.to_string())?;
            guard.prepend_callback_continuations(modes);
        }
        guard
            .leave_mode_callback(entered)
            .map_err(|error| error.to_string())?;
    }
}

fn drain_tick_notifications(
    runtime: &DbcRuntime,
    event_callback: Option<DbcEventCallback>,
    context: *mut std::ffi::c_void,
) -> Result<(), String> {
    let mut pass = runtime
        .runtime
        .lock()
        .map_err(|_| "runtime data lock is poisoned")?
        .begin_notification_pass();
    loop {
        let event = runtime
            .runtime
            .lock()
            .map_err(|_| "runtime data lock is poisoned")?
            .next_tick_notification(&mut pass)
            .map_err(|error| error.to_string())?;
        let Some(event) = event else {
            return Ok(());
        };
        // Only this operation owns the continuation. Reentrant calls finish
        // their own work before this one resumes and reads the live state.
        let mut one = VecDeque::from([event]);
        drain_callbacks(
            runtime,
            &mut VecDeque::new(),
            &mut one,
            None,
            event_callback,
            context,
        )?;
    }
}

fn drain_motion_continuations(
    runtime: &DbcRuntime,
    callback: Option<DbcModeCallback>,
    event_callback: Option<DbcEventCallback>,
    context: *mut std::ffi::c_void,
) -> Result<(), String> {
    let mut callbacks = VecDeque::new();
    {
        let mut guard = runtime
            .runtime
            .lock()
            .map_err(|_| "runtime data lock is poisoned")?;
        while let Some(event) = guard.take_mode_callback() {
            callbacks.push_back(event);
        }
    }
    drain_callbacks(
        runtime,
        &mut callbacks,
        &mut VecDeque::new(),
        callback,
        event_callback,
        context,
    )
}

fn runtime_call_inner(
    runtime: *mut DbcRuntime,
    request_json: *const u8,
    request_len: usize,
    callback: Option<DbcModeCallback>,
    event_callback: Option<DbcEventCallback>,
    context: *mut std::ffi::c_void,
) -> DbcBuffer {
    if runtime.is_null() {
        return response_buffer(
            json!({"ok": false, "error": {"code": "invalid_runtime", "message": "runtime is null"}}),
            1,
        );
    }
    let bytes = match input_bytes(request_json, request_len) {
        Ok(bytes) => bytes,
        Err(message) => {
            return response_buffer(
                json!({"ok": false, "error": {"code": "invalid_request", "message": message}}),
                1,
            );
        }
    };
    let request: CompatRequest = match strict_json_from_slice(&bytes) {
        Ok(request) => request,
        Err(error) => {
            return response_buffer(
                json!({"ok": false, "error": {"code": "invalid_request", "message": format!("invalid request JSON: {error}")}}),
                1,
            );
        }
    };
    // SAFETY: null was rejected above. The caller keeps the runtime alive for
    // the duration of this synchronous call.
    let runtime = unsafe { &*runtime };
    if runtime.poisoned.load(Ordering::Acquire) {
        return poisoned_response();
    }
    let mut guard = match runtime.runtime.lock() {
        Ok(guard) => guard,
        Err(_) => {
            runtime.poisoned.store(true, Ordering::Release);
            return poisoned_response();
        }
    };
    let dispatched = guard.dispatch_yielding(request);
    drop(guard);
    match dispatched {
        Ok((result, finish_tick, mut callbacks)) => {
            // This operation owns exactly the notifications it produced. A
            // nested read cannot pump a pending batch belonging to its caller.
            if let Err(message) = drain_callbacks(
                runtime,
                &mut callbacks,
                &mut VecDeque::new(),
                callback,
                event_callback,
                context,
            ) {
                runtime.poisoned.store(true, Ordering::Release);
                return response_buffer(
                    json!({"ok":false,"error":{"code":"panic","message":message}}),
                    2,
                );
            }
            if finish_tick {
                let mut guard = match runtime.runtime.lock() {
                    Ok(guard) => guard,
                    Err(_) => return poisoned_response(),
                };
                if let Err(error) = guard.advance_callback_tick() {
                    return response_buffer(json!({"ok":false,"error":error.payload()}), 0);
                }
                drop(guard);
                if let Err(message) =
                    drain_motion_continuations(runtime, callback, event_callback, context)
                {
                    runtime.poisoned.store(true, Ordering::Release);
                    return response_buffer(
                        json!({"ok":false,"error":{"code":"panic","message":message}}),
                        2,
                    );
                }
                if let Err(message) = drain_tick_notifications(runtime, event_callback, context) {
                    runtime.poisoned.store(true, Ordering::Release);
                    return response_buffer(
                        json!({"ok":false,"error":{"code":"panic","message":message}}),
                        2,
                    );
                }
                let mut guard = match runtime.runtime.lock() {
                    Ok(guard) => guard,
                    Err(_) => return poisoned_response(),
                };
                if let Err(error) = guard.finish_callback_tick() {
                    return response_buffer(json!({"ok":false,"error":error.payload()}), 0);
                }
            }
            response_buffer(json!({"ok": true, "result": result}), 0)
        }
        Err(error) => response_buffer(json!({"ok": false, "error": error.payload()}), 0),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_runtime_update(runtime: *mut DbcRuntime) -> DbcBuffer {
    dbc_runtime_update_with_mode_callback(runtime, None, ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_runtime_update_with_mode_callback(
    runtime: *mut DbcRuntime,
    callback: Option<DbcModeCallback>,
    context: *mut std::ffi::c_void,
) -> DbcBuffer {
    runtime_update_with_callbacks(runtime, callback, None, context)
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_runtime_update_with_event_callback(
    runtime: *mut DbcRuntime,
    callback: Option<DbcEventCallback>,
    context: *mut std::ffi::c_void,
) -> DbcBuffer {
    runtime_update_with_callbacks(runtime, None, callback, context)
}

fn runtime_update_with_callbacks(
    runtime: *mut DbcRuntime,
    callback: Option<DbcModeCallback>,
    event_callback: Option<DbcEventCallback>,
    context: *mut std::ffi::c_void,
) -> DbcBuffer {
    protected_runtime_operation(runtime, || {
        if runtime.is_null() {
            return response_buffer(
                json!({"ok": false, "error": {"code": "invalid_runtime", "message": "runtime is null"}}),
                1,
            );
        }
        // SAFETY: null was rejected and this is a synchronous call.
        let runtime = unsafe { &*runtime };
        if runtime.poisoned.load(Ordering::Acquire) {
            return poisoned_response();
        }
        let mut guard = match runtime.runtime.lock() {
            Ok(guard) => guard,
            Err(_) => {
                runtime.poisoned.store(true, Ordering::Release);
                return poisoned_response();
            }
        };
        let update = guard.begin_callback_tick(false);
        drop(guard);
        let result =
            update.and_then(|()| {
                let mut modes = {
                    let mut guard = runtime.runtime.lock().map_err(|_| {
                        crate::runtime::CompatError::Engine("runtime lock poisoned".into())
                    })?;
                    let mut modes = VecDeque::new();
                    while let Some(event) = guard.take_mode_callback() {
                        modes.push_back(event);
                    }
                    modes
                };
                drain_callbacks(
                    runtime,
                    &mut modes,
                    &mut VecDeque::new(),
                    callback,
                    event_callback,
                    context,
                )
                .map_err(crate::runtime::CompatError::Engine)?;
                let mut guard = runtime.runtime.lock().map_err(|_| {
                    crate::runtime::CompatError::Engine("runtime lock poisoned".into())
                })?;
                guard.advance_callback_tick()?;
                drop(guard);
                drain_motion_continuations(runtime, callback, event_callback, context)
                    .map_err(crate::runtime::CompatError::Engine)?;
                drain_tick_notifications(runtime, event_callback, context)
                    .map_err(crate::runtime::CompatError::Engine)?;
                let mut guard = runtime.runtime.lock().map_err(|_| {
                    crate::runtime::CompatError::Engine("runtime lock poisoned".into())
                })?;
                guard.finish_callback_tick()?;
                Ok(json!({"current_time":guard.current_time()}))
            });
        match result {
            Ok(result) => response_buffer(json!({"ok": true, "result": result}), 0),
            Err(error) => response_buffer(json!({"ok": false, "error": error.payload()}), 0),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_buffer_free(buffer: DbcBuffer) {
    if buffer.data.is_null() || buffer.len == 0 {
        return;
    }
    // SAFETY: buffer_from_bytes allocated this exact boxed slice and ownership
    // is transferred to this function exactly once by the ABI contract.
    unsafe {
        let slice = ptr::slice_from_raw_parts_mut(buffer.data, buffer.len);
        drop(Box::from_raw(slice));
    }
}

fn input_bytes(data: *const u8, len: usize) -> Result<Vec<u8>, String> {
    if len > MAX_REQUEST_BYTES {
        return Err(format!(
            "input exceeds the {MAX_REQUEST_BYTES}-byte request limit"
        ));
    }
    if len == 0 {
        return Ok(Vec::new());
    }
    if data.is_null() {
        return Err("input pointer is null while length is non-zero".into());
    }
    // SAFETY: the caller guarantees that data points to len readable bytes for
    // this synchronous call; null was checked above.
    Ok(unsafe { slice::from_raw_parts(data, len) }.to_vec())
}

fn response_buffer(value: Value, status: i32) -> DbcBuffer {
    let mut writer = LimitedWriter::new(MAX_RESPONSE_BYTES);
    match serde_json::to_writer(&mut writer, &value) {
        Ok(()) => buffer_from_bytes(writer.into_bytes(), status),
        Err(_) if writer.overflowed => buffer_from_bytes(
            br#"{"ok":false,"error":{"code":"response_too_large","message":"response exceeds the ABI byte limit"}}"#.to_vec(),
            1,
        ),
        Err(_) => buffer_from_bytes(
            br#"{"ok":false,"error":{"code":"serialization_error","message":"response serialization failed"}}"#.to_vec(),
            2,
        ),
    }
}

struct LimitedWriter {
    bytes: Vec<u8>,
    limit: usize,
    overflowed: bool,
}

impl LimitedWriter {
    fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
            overflowed: false,
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

impl Write for LimitedWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let Some(new_len) = self.bytes.len().checked_add(buffer.len()) else {
            self.overflowed = true;
            return Err(io::Error::other("response byte limit exceeded"));
        };
        if new_len > self.limit {
            self.overflowed = true;
            return Err(io::Error::other("response byte limit exceeded"));
        }
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
fn mark_poisoned(runtime: *mut DbcRuntime) {
    if !runtime.is_null() {
        // SAFETY: panic handling occurs during a synchronous call for which the
        // caller is required to retain the runtime.
        unsafe { &*runtime }.poisoned.store(true, Ordering::Release);
    }
}

fn poisoned_response() -> DbcBuffer {
    response_buffer(
        json!({
            "ok": false,
            "error": {
                "code": "runtime_poisoned",
                "message": "the runtime previously panicked and cannot safely continue"
            }
        }),
        2,
    )
}

fn buffer_from_bytes(bytes: Vec<u8>, status: i32) -> DbcBuffer {
    if bytes.is_empty() {
        return DbcBuffer::empty(status);
    }
    let mut boxed = bytes.into_boxed_slice();
    let len = boxed.len();
    let data = boxed.as_mut_ptr();
    std::mem::forget(boxed);
    DbcBuffer { data, len, status }
}

fn diagnostic_buffer(message: String, status: i32) -> DbcBuffer {
    let bytes = message.into_bytes();
    if bytes.len() > MAX_RESPONSE_BYTES {
        return buffer_from_bytes(
            b"runtime diagnostic exceeds the ABI byte limit".to_vec(),
            status,
        );
    }
    buffer_from_bytes(bytes, status)
}

fn set_out_error(target: *mut DbcBuffer, value: DbcBuffer) {
    if target.is_null() {
        // The optional diagnostic has no owner, so reclaim it immediately.
        dbc_buffer_free(value);
        return;
    }
    // SAFETY: caller supplied an optional writable DbcBuffer pointer.
    unsafe {
        target.write(value);
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "panic crossed the runtime boundary and was contained".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn consume_json(buffer: DbcBuffer) -> Value {
        assert!(!buffer.data.is_null());
        assert!(buffer.len > 0);
        // SAFETY: the buffer was returned by this module and remains owned by
        // this test until the copy completes immediately below.
        let bytes = unsafe { slice::from_raw_parts(buffer.data, buffer.len) }.to_vec();
        dbc_buffer_free(buffer);
        serde_json::from_slice(&bytes).expect("ABI JSON response")
    }

    #[test]
    fn input_pointer_and_empty_buffer_contracts_fail_safely() {
        assert!(
            input_bytes(ptr::null(), 0)
                .expect("zero-length null input")
                .is_empty()
        );
        assert!(input_bytes(ptr::null(), 1).is_err());
        assert!(input_bytes(ptr::null(), MAX_REQUEST_BYTES + 1).is_err());

        // Empty/null buffers own no allocation, so repeated no-op frees are
        // well-defined. Re-freeing a nonempty owned buffer remains explicitly
        // forbidden by the public ownership contract.
        let empty = DbcBuffer::empty(0);
        dbc_buffer_free(empty);
        dbc_buffer_free(empty);
        dbc_buffer_free(DbcBuffer {
            data: ptr::null_mut(),
            len: 7,
            status: 1,
        });
    }

    #[test]
    fn runtime_retain_release_and_poison_boundaries_are_contained() {
        let mut error = DbcBuffer::empty(0);
        let runtime = dbc_runtime_new(ptr::null(), 0, &mut error);
        assert!(!runtime.is_null());
        assert!(error.data.is_null());
        assert_eq!(error.status, 0);

        assert!(dbc_runtime_retain(runtime));
        dbc_runtime_release(runtime);

        let malformed = br#"{"title":"x","title":"y"}"#;
        let response = dbc_runtime_call(runtime, malformed.as_ptr(), malformed.len());
        assert_eq!(response.status, 1);
        let payload = consume_json(response);
        assert_eq!(payload["ok"], json!(false));

        mark_poisoned(runtime);
        let response = dbc_runtime_update(runtime);
        assert_eq!(response.status, 2);
        let payload = consume_json(response);
        assert_eq!(payload["error"]["code"], json!("runtime_poisoned"));

        dbc_runtime_release(runtime);
        assert!(!dbc_runtime_retain(ptr::null_mut()));
        dbc_runtime_release(ptr::null_mut());
    }

    #[test]
    fn retain_rejects_the_machine_limit_without_panicking_or_changing_count() {
        let runtime = dbc_runtime_new(ptr::null(), 0, ptr::null_mut());
        assert!(!runtime.is_null());
        // Exercise the actual ABI's machine boundary without allocating
        // usize::MAX aliases. This does not discharge callers' live ownership.
        let reference = unsafe { &*runtime };
        reference.references.store(usize::MAX, Ordering::SeqCst);
        assert!(!dbc_runtime_retain(runtime));
        assert_eq!(reference.references.load(Ordering::SeqCst), usize::MAX);
        reference.references.store(1, Ordering::SeqCst);
        dbc_runtime_release(runtime);
    }

    #[test]
    fn independently_owned_thread_references_preserve_the_creator_lease() {
        use std::sync::{Arc, Barrier};
        let runtime = dbc_runtime_new(ptr::null(), 0, ptr::null_mut());
        assert!(!runtime.is_null());
        let address = runtime as usize;
        let start = Arc::new(Barrier::new(5));
        let mut workers = Vec::new();
        for _ in 0..4 {
            assert!(dbc_runtime_retain(runtime)); // each worker receives its own live lease
            let start = Arc::clone(&start);
            workers.push(std::thread::spawn(move || {
                let runtime = address as *mut DbcRuntime;
                start.wait();
                for _ in 0..512 {
                    assert!(dbc_runtime_retain(runtime));
                    dbc_runtime_release(runtime);
                }
                dbc_runtime_release(runtime); // transfer back the worker's one lease
            }));
        }
        start.wait();
        for worker in workers {
            worker.join().expect("reference-count worker");
        }
        assert_eq!(unsafe { &*runtime }.references.load(Ordering::SeqCst), 1);
        // The Bevy App remains on its creator thread; this exercises actual
        // concurrent retain/release, without claiming cross-thread ECS updates.
        dbc_runtime_release(runtime);
    }
}
