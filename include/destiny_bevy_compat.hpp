// C++17 RAII facade for the stable Destiny-to-Bevy C ABI.
#ifndef DESTINY_BEVY_COMPAT_HPP
#define DESTINY_BEVY_COMPAT_HPP

#include "destiny_bevy_compat.h"

#include <cstdint>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>

namespace dbc {

class Error : public std::runtime_error {
public:
    using std::runtime_error::runtime_error;
};

class Buffer final {
public:
    Buffer() noexcept = default;
    explicit Buffer(DbcBuffer raw) noexcept : raw_(raw) {}

    ~Buffer() { reset(); }

    Buffer(const Buffer&) = delete;
    Buffer& operator=(const Buffer&) = delete;

    Buffer(Buffer&& other) noexcept : raw_(other.release()) {}

    Buffer& operator=(Buffer&& other) noexcept {
        if (this != &other) {
            reset();
            raw_ = other.release();
        }
        return *this;
    }

    [[nodiscard]] bool valid() const noexcept {
        return (raw_.data == nullptr) == (raw_.len == 0);
    }
    [[nodiscard]] bool empty() const noexcept { return valid() && raw_.data == nullptr; }
    [[nodiscard]] const std::uint8_t* data() const noexcept { return raw_.data; }
    [[nodiscard]] std::size_t size() const noexcept { return raw_.len; }
    [[nodiscard]] std::int32_t status() const noexcept { return raw_.status; }

    [[nodiscard]] std::string string() const {
        if (!valid()) {
            throw Error("Destiny-to-Bevy ABI returned an invalid pointer/length pair");
        }
        if (empty()) {
            return {};
        }
        return std::string(reinterpret_cast<const char*>(raw_.data), raw_.len);
    }

    void reset() noexcept {
        if (raw_.data != nullptr) {
            dbc_buffer_free(raw_);
        }
        raw_ = {};
    }

    [[nodiscard]] DbcBuffer release() noexcept {
        DbcBuffer result = raw_;
        raw_ = {};
        return result;
    }

private:
    DbcBuffer raw_{};
};

class Runtime final {
public:
    explicit Runtime(std::string_view options_json = "{}") {
        const auto actual = dbc_abi_version();
        if (actual != DBC_ABI_VERSION) {
            throw Error("Destiny-to-Bevy ABI version mismatch");
        }
        DbcBuffer error{};
        runtime_ = dbc_runtime_new(
            reinterpret_cast<const std::uint8_t*>(options_json.data()),
            options_json.size(),
            &error
        );
        Buffer owned_error(error);
        if (runtime_ == nullptr) {
            throw Error(owned_error.empty() ? "failed to create Bevy compatibility runtime" : owned_error.string());
        }
        if (!owned_error.valid() || owned_error.status() != 0 || !owned_error.empty()) {
            dbc_runtime_release(runtime_);
            runtime_ = nullptr;
            throw Error(
                owned_error.valid() && !owned_error.empty()
                    ? owned_error.string()
                    : "Bevy compatibility runtime returned an invalid success diagnostic"
            );
        }
    }

    ~Runtime() { dbc_runtime_release(runtime_); }

    Runtime(const Runtime&) = delete;
    Runtime& operator=(const Runtime&) = delete;

    Runtime(Runtime&& other) noexcept : runtime_(std::exchange(other.runtime_, nullptr)) {}

    Runtime& operator=(Runtime&& other) noexcept {
        if (this != &other) {
            dbc_runtime_release(runtime_);
            runtime_ = std::exchange(other.runtime_, nullptr);
        }
        return *this;
    }

    [[nodiscard]] Buffer call(std::string_view request_json) {
        ensure_open();
        return Buffer(dbc_runtime_call(
            runtime_,
            reinterpret_cast<const std::uint8_t*>(request_json.data()),
            request_json.size()
        ));
    }

    [[nodiscard]] std::string call_json(std::string_view request_json) {
        Buffer response = call(request_json);
        if (response.status() != 0) {
            throw Error(response.empty() ? "Bevy compatibility call failed" : response.string());
        }
        return response.string();
    }

    [[nodiscard]] Buffer update() {
        ensure_open();
        return Buffer(dbc_runtime_update(runtime_));
    }

    [[nodiscard]] DbcRuntime* native_handle() const noexcept { return runtime_; }

private:
    void ensure_open() const {
        if (runtime_ == nullptr) {
            throw Error("Bevy compatibility runtime is closed");
        }
    }

    DbcRuntime* runtime_ = nullptr;
};

} // namespace dbc

#endif // DESTINY_BEVY_COMPAT_HPP
