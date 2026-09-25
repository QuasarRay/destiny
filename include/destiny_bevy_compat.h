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

/** Release bytes returned by this ABI. A zero/null buffer is allowed. */
DBC_API void dbc_buffer_free(DbcBuffer buffer);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* DESTINY_BEVY_COMPAT_H */
