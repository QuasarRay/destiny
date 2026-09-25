use std::{
    any::Any,
    io::{self, Write},
    panic::{AssertUnwindSafe, catch_unwind},
    ptr, slice,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use serde_json::{Value, json};

use crate::runtime::{CompatOptions, CompatRequest, CompatRuntime, strict_json_from_slice};

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
        })))
    }));
    match result {
        Ok(Ok(runtime)) => runtime,
        Ok(Err(message)) => {
            set_out_error(out_error, diagnostic_buffer(message, 1));
            ptr::null_mut()
        }
        Err(payload) => {
            set_out_error(
                out_error,
                diagnostic_buffer(panic_message(payload), 2),
            );
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
    runtime
        .references
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
            (count > 0 && count < usize::MAX).then_some(count + 1)
        })
        .is_ok()
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_runtime_release(runtime: *mut DbcRuntime) {
    if runtime.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: the caller transfers one live reference to this function.
        let runtime_ref = unsafe { &*runtime };
        let previous = runtime_ref.references.fetch_sub(1, Ordering::AcqRel);
        if previous == 0 {
            runtime_ref.references.store(0, Ordering::Release);
            return;
        }
        if previous == 1 {
            // Acquire pairs with every previous release before final teardown.
            std::sync::atomic::fence(Ordering::Acquire);
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
    match catch_unwind(AssertUnwindSafe(|| {
        runtime_call_inner(runtime, request_json, request_len)
    })) {
        Ok(buffer) => buffer,
        Err(payload) => {
            mark_poisoned(runtime);
            response_buffer(
                json!({"ok": false, "error": {"code": "panic", "message": panic_message(payload)}}),
                2,
            )
        }
    }
}

fn runtime_call_inner(
    runtime: *mut DbcRuntime,
    request_json: *const u8,
    request_len: usize,
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
    match guard.dispatch(request) {
        Ok(result) => response_buffer(json!({"ok": true, "result": result}), 0),
        Err(error) => response_buffer(json!({"ok": false, "error": error.payload()}), 0),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dbc_runtime_update(runtime: *mut DbcRuntime) -> DbcBuffer {
    match catch_unwind(AssertUnwindSafe(|| {
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
        match guard.update() {
            Ok(result) => response_buffer(json!({"ok": true, "result": result}), 0),
            Err(error) => response_buffer(json!({"ok": false, "error": error.payload()}), 0),
        }
    })) {
        Ok(buffer) => buffer,
        Err(payload) => {
            mark_poisoned(runtime);
            response_buffer(
                json!({"ok": false, "error": {"code": "panic", "message": panic_message(payload)}}),
                2,
            )
        }
    }
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
        assert!(input_bytes(ptr::null(), 0).expect("zero-length null input").is_empty());
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
}
