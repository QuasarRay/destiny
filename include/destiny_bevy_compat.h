/* Stable C ABI for Destiny-to-Bevy compatibility. */
#ifndef DESTINY_BEVY_COMPAT_H
#define DESTINY_BEVY_COMPAT_H

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>

#if defined(_WIN32) && defined(DBC_BUILD_SHARED)
#  if defined(DBC_BUILDING_LIBRARY)
#    define DBC_API __declspec(dllexport)
#  else
#    define DBC_API __declspec(dllimport)
#  endif
#elif defined(__GNUC__) && defined(DBC_BUILD_SHARED)
#  define DBC_API __attribute__((visibility("default")))
#else
#  define DBC_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

#define DBC_ABI_VERSION 1u

/** Opaque owner of one headless Bevy App/World and its Avian simulation. */
typedef struct DbcRuntime DbcRuntime;

/**
 * ABI-owned bytes. When data is non-null, release exactly once with
 * dbc_buffer_free, even when status is non-zero.
 */
typedef struct DbcBuffer {
    uint8_t* data;
    size_t len;
    int32_t status;
} DbcBuffer;

/** Return the ABI version compiled into the library. */
DBC_API uint32_t dbc_abi_version(void);

/**
 * Create a runtime. options_json may be null when options_len is zero.
 * out_error is optional and may be null. When non-null, it must point to a
 * writable, zero-initialized DbcBuffer with no outstanding owned data. On
 * failure, the function returns null and writes a UTF-8 diagnostic there.
 */
DBC_API DbcRuntime* dbc_runtime_new(
    const uint8_t* options_json,
    size_t options_len,
    DbcBuffer* out_error
);

/**
 * Release one runtime reference. This is the backward-compatible spelling of
 * dbc_runtime_release. Passing null is allowed.
 *
 * A caller must own a live reference for the full duration of every call or
 * update. The final release must not race a thread that has not first retained
 * its own reference.
 */
DBC_API void dbc_runtime_free(DbcRuntime* runtime);

/**
 * Acquire one additional reference before sharing a runtime with another
 * owner/thread. Returns false for null or a reference-count overflow.
 */
DBC_API bool dbc_runtime_retain(DbcRuntime* runtime);

/** Release one reference acquired by new or retain. Passing null is allowed. */
DBC_API void dbc_runtime_release(DbcRuntime* runtime);

/**
 * Dispatch one UTF-8 JSON request. The UTF-8 JSON response always has an
 * `ok` field. status is zero when the request could be decoded and dispatched.
 */
DBC_API DbcBuffer dbc_runtime_call(
    DbcRuntime* runtime,
    const uint8_t* request_json,
    size_t request_len
);

/** Advance the Bevy application once and return a JSON result envelope. */
DBC_API DbcBuffer dbc_runtime_update(DbcRuntime* runtime);

/** Synchronous mode notification, before the ball stores its new mode.
 * The callback may reenter this runtime on the calling thread. An evolution
 * retains exclusive operation ownership and rejects AddBall until it finishes.
 * No runtime data lock is held across the callback. It must return without
 * unwinding/longjmp, and its context must remain live for the enclosing call.
 * Each enclosing call retains an internal reference until all callbacks and
 * cleanup finish, including when a callback releases the caller's reference.
 */
typedef void (*DbcModeCallback)(void* context, int64_t ball_id, uint8_t old_mode, uint8_t new_mode);
DBC_API DbcBuffer dbc_runtime_call_with_mode_callback(
    DbcRuntime* runtime, const uint8_t* request_json, size_t request_len,
    DbcModeCallback callback, void* context
);
DBC_API DbcBuffer dbc_runtime_update_with_mode_callback(
    DbcRuntime* runtime, DbcModeCallback callback, void* context
);

/** General synchronous event notification. Arguments are a UTF-8 JSON array,
 * read-only and valid only until this callback returns. event_kind uses the
 * Destiny event constants (3 proximity, 7 range, 8 mode change); extension kind
 * 13 denotes target tracking. Extension kind 14 targets the park (ball_id=0)
 * with [transitions, dictionaries]: transitions is an ordered array of
 * [ball_id, old_bubble, new_bubble]; dictionaries is four ordered arrays of
 * [signed_id, value] pairs, whose values are a signed bubble ID or an ordered
 * array of signed ball IDs. The Python adapter publishes those dictionary
 * mutations before invoking DoBubbleTransitions(transitions). Extension kind 15
 * denotes DoFinalCleanup, with an empty argument array, while the moribund ball
 * remains in the park. The ownership, reentry
 * and no-foreign-unwind rules of DbcModeCallback also apply.
 */
typedef void (*DbcEventCallback)(void* context, int64_t ball_id, uint32_t event_kind,
                               const uint8_t* arguments_json, size_t arguments_len);
DBC_API DbcBuffer dbc_runtime_call_with_event_callback(
    DbcRuntime* runtime, const uint8_t* request_json, size_t request_len,
    DbcEventCallback callback, void* context
);
DBC_API DbcBuffer dbc_runtime_update_with_event_callback(
    DbcRuntime* runtime, DbcEventCallback callback, void* context
);

/** Release bytes returned by this ABI. A zero/null buffer is allowed. */
DBC_API void dbc_buffer_free(DbcBuffer buffer);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* DESTINY_BEVY_COMPAT_H */
