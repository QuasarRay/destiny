// Destiny-named C++17 convenience facade for Carbon native consumers.
#ifndef DESTINY_CARBON_HPP
#define DESTINY_CARBON_HPP

#include "destiny_bevy_compat.hpp"

#include <charconv>
#include <cmath>
#include <cstdint>
#include <iomanip>
#include <locale>
#include <memory>
#include <mutex>
#include <optional>
#include <sstream>
#include <string>
#include <string_view>
#include <system_error>
#include <utility>

namespace destiny {

namespace detail {

inline std::string number(double value) {
    if (!std::isfinite(value)) {
        throw dbc::Error("Destiny numeric arguments must be finite");
    }
    std::ostringstream stream;
    stream.imbue(std::locale::classic());
    stream << std::setprecision(17) << value;
    return stream.str();
}

inline const char* boolean(bool value) noexcept { return value ? "true" : "false"; }

inline std::string json_string(std::string_view value) {
    static constexpr char hex[] = "0123456789abcdef";
    std::string result;
    result.reserve(value.size() + 2);
    result.push_back('"');
    for (const unsigned char byte : value) {
        switch (byte) {
        case '"': result += "\\\""; break;
        case '\\': result += "\\\\"; break;
        case '\b': result += "\\b"; break;
        case '\f': result += "\\f"; break;
        case '\n': result += "\\n"; break;
        case '\r': result += "\\r"; break;
        case '\t': result += "\\t"; break;
        default:
            if (byte < 0x20) {
                result += "\\u00";
                result.push_back(hex[byte >> 4]);
                result.push_back(hex[byte & 0x0f]);
            } else {
                result.push_back(static_cast<char>(byte));
            }
        }
    }
    result.push_back('"');
    return result;
}

class JsonCursor final {
public:
    explicit JsonCursor(std::string_view input) noexcept : input_(input) {}

    void whitespace() noexcept {
        while (position_ < input_.size()) {
            const char value = input_[position_];
            if (value != ' ' && value != '\n' && value != '\r' && value != '\t') break;
            ++position_;
        }
    }

    bool consume(char expected) noexcept {
        whitespace();
        if (position_ < input_.size() && input_[position_] == expected) {
            ++position_;
            return true;
        }
        return false;
    }

    [[nodiscard]] bool finished() noexcept {
        whitespace();
        return position_ == input_.size();
    }

    std::string string() {
        whitespace();
        if (position_ >= input_.size() || input_[position_++] != '"') {
            throw dbc::Error("Malformed Destiny Bevy JSON string");
        }
        std::string result;
        while (position_ < input_.size()) {
            const unsigned char value = static_cast<unsigned char>(input_[position_++]);
            if (value == '"') return result;
            if (value < 0x20) throw dbc::Error("Malformed Destiny Bevy JSON control character");
            if (value != '\\') {
                result.push_back(static_cast<char>(value));
                continue;
            }
            if (position_ >= input_.size()) throw dbc::Error("Malformed Destiny Bevy JSON escape");
            const char escaped = input_[position_++];
            switch (escaped) {
            case '"': case '\\': case '/': result.push_back(escaped); break;
            case 'b': result.push_back('\b'); break;
            case 'f': result.push_back('\f'); break;
            case 'n': result.push_back('\n'); break;
            case 'r': result.push_back('\r'); break;
            case 't': result.push_back('\t'); break;
            case 'u':
                // ABI envelope keys are ASCII. Validate the escape but retain
                // a sentinel so an escaped spelling cannot match a known key.
                for (int index = 0; index < 4; ++index) {
                    if (position_ >= input_.size() || !is_hex(input_[position_++])) {
                        throw dbc::Error("Malformed Destiny Bevy JSON unicode escape");
                    }
                }
                result.push_back('?');
                break;
            default: throw dbc::Error("Malformed Destiny Bevy JSON escape");
            }
        }
        throw dbc::Error("Unterminated Destiny Bevy JSON string");
    }

    std::string_view value() {
        whitespace();
        const std::size_t begin = position_;
        skip_value(0);
        return input_.substr(begin, position_ - begin);
    }

private:
    static bool is_hex(char value) noexcept {
        return (value >= '0' && value <= '9') || (value >= 'a' && value <= 'f') ||
            (value >= 'A' && value <= 'F');
    }

    void literal(std::string_view expected) {
        if (input_.substr(position_, expected.size()) != expected) {
            throw dbc::Error("Malformed Destiny Bevy JSON literal");
        }
        position_ += expected.size();
    }

    void skip_string() { (void)string(); }

    void skip_number() {
        const std::size_t begin = position_;
        if (position_ < input_.size() && input_[position_] == '-') ++position_;
        if (position_ >= input_.size()) throw dbc::Error("Malformed Destiny Bevy JSON number");
        if (input_[position_] == '0') {
            ++position_;
        } else {
            if (input_[position_] < '1' || input_[position_] > '9') {
                throw dbc::Error("Malformed Destiny Bevy JSON number");
            }
            while (position_ < input_.size() && input_[position_] >= '0' && input_[position_] <= '9') {
                ++position_;
            }
        }
        if (position_ < input_.size() && input_[position_] == '.') {
            ++position_;
            const std::size_t digits = position_;
            while (position_ < input_.size() && input_[position_] >= '0' && input_[position_] <= '9') {
                ++position_;
            }
            if (position_ == digits) throw dbc::Error("Malformed Destiny Bevy JSON fraction");
        }
        if (position_ < input_.size() && (input_[position_] == 'e' || input_[position_] == 'E')) {
            ++position_;
            if (position_ < input_.size() && (input_[position_] == '+' || input_[position_] == '-')) ++position_;
            const std::size_t digits = position_;
            while (position_ < input_.size() && input_[position_] >= '0' && input_[position_] <= '9') {
                ++position_;
            }
            if (position_ == digits) throw dbc::Error("Malformed Destiny Bevy JSON exponent");
        }
        if (position_ == begin) throw dbc::Error("Malformed Destiny Bevy JSON number");
    }

    void skip_value(unsigned depth) {
        if (depth > 128) throw dbc::Error("Destiny Bevy JSON nesting limit exceeded");
        whitespace();
        if (position_ >= input_.size()) throw dbc::Error("Missing Destiny Bevy JSON value");
        switch (input_[position_]) {
        case '"': skip_string(); return;
        case 't': literal("true"); return;
        case 'f': literal("false"); return;
        case 'n': literal("null"); return;
        case '[':
            ++position_;
            if (consume(']')) return;
            for (;;) {
                skip_value(depth + 1);
                if (consume(']')) return;
                if (!consume(',')) throw dbc::Error("Malformed Destiny Bevy JSON array");
            }
        case '{':
            ++position_;
            if (consume('}')) return;
            for (;;) {
                skip_string();
                if (!consume(':')) throw dbc::Error("Malformed Destiny Bevy JSON object");
                skip_value(depth + 1);
                if (consume('}')) return;
                if (!consume(',')) throw dbc::Error("Malformed Destiny Bevy JSON object");
            }
        default: skip_number(); return;
        }
    }

    std::string_view input_;
    std::size_t position_ = 0;
};

inline std::string result_from_response(const std::string& response) {
    JsonCursor cursor(response);
    if (!cursor.consume('{')) throw dbc::Error("Malformed Destiny Bevy response envelope");
    bool seen_ok = false;
    bool ok = false;
    bool seen_result = false;
    bool seen_error = false;
    std::string result;
    if (!cursor.consume('}')) {
        for (;;) {
            const std::string key = cursor.string();
            if (!cursor.consume(':')) throw dbc::Error("Malformed Destiny Bevy response member");
            const std::string_view value = cursor.value();
            if (key == "ok") {
                if (seen_ok || (value != "true" && value != "false")) {
                    throw dbc::Error("Malformed Destiny Bevy ok field");
                }
                seen_ok = true;
                ok = value == "true";
            } else if (key == "result") {
                if (seen_result) throw dbc::Error("Duplicate Destiny Bevy result field");
                seen_result = true;
                result.assign(value.data(), value.size());
            } else if (key == "error") {
                if (seen_error) throw dbc::Error("Duplicate Destiny Bevy error field");
                seen_error = true;
            } else {
                throw dbc::Error("Unknown Destiny Bevy response field");
            }
            if (cursor.consume('}')) break;
            if (!cursor.consume(',')) throw dbc::Error("Malformed Destiny Bevy response object");
        }
    }
    if (!cursor.finished() || !seen_ok) throw dbc::Error("Malformed Destiny Bevy response envelope");
    if (ok && (!seen_result || seen_error)) {
        throw dbc::Error("Malformed Destiny Bevy success envelope");
    }
    if (!ok && (!seen_error || seen_result)) {
        throw dbc::Error("Malformed Destiny Bevy error envelope");
    }
    if (!ok) throw dbc::Error(response.empty() ? "Destiny Bevy call failed" : response);
    return result;
}

inline void require_json_value(std::string_view value, const char* name) {
    JsonCursor cursor(value);
    (void)cursor.value();
    if (!cursor.finished()) {
        throw dbc::Error(std::string(name) + " must contain exactly one JSON value");
    }
}

inline double exact_double(const std::string& value) {
    std::istringstream stream(value);
    stream.imbue(std::locale::classic());
    stream >> std::noskipws;
    double result = 0.0;
    char extra = '\0';
    if (!(stream >> result) || (stream >> extra) || !std::isfinite(result)) {
        throw dbc::Error("Destiny Bevy result is not one finite JSON number");
    }
    return result;
}

inline std::optional<double> optional_double(const std::string& value) {
    if (value == "null") return std::nullopt;
    return exact_double(value);
}

inline std::int64_t exact_integer(const std::string& value) {
    std::int64_t result = 0;
    const char* begin = value.data();
    const char* end = begin + value.size();
    const auto parsed = std::from_chars(begin, end, result, 10);
    if (parsed.ec != std::errc{} || parsed.ptr != end) {
        throw dbc::Error("Destiny Bevy result is not one signed integer");
    }
    return result;
}

inline bool exact_boolean(const std::string& value) {
    if (value == "true") return true;
    if (value == "false") return false;
    throw dbc::Error("Destiny Bevy result is not a boolean");
}

class RuntimeState final {
public:
    RuntimeState() : runtime_("{}") {}

    std::string call(
        std::string_view title,
        std::string_view operation,
        std::string_view target_json,
        std::string_view args_json
    ) {
        require_json_value(target_json, "Destiny target_json");
        require_json_value(args_json, "Destiny args_json");
        const std::string request = std::string("{\"title\":") + json_string(title) +
            ",\"operation\":" + json_string(operation) + ",\"target\":" +
            std::string(target_json) + ",\"args\":" + std::string(args_json) + ",\"kwargs\":{}}";
        std::lock_guard<std::mutex> lock(mutex_);
        return result_from_response(runtime_.call_json(request));
    }

    std::string update() {
        std::lock_guard<std::mutex> lock(mutex_);
        dbc::Buffer response = runtime_.update();
        if (response.status() != 0) {
            throw dbc::Error(response.empty() ? "Destiny Bevy update failed" : response.string());
        }
        return result_from_response(response.string());
    }

private:
    std::mutex mutex_;
    dbc::Runtime runtime_;
};

} // namespace detail

class Ball final {
public:
    [[nodiscard]] std::int64_t id() const noexcept { return id_; }
    [[nodiscard]] double x() const;
    [[nodiscard]] double y() const;
    [[nodiscard]] double z() const;
    [[nodiscard]] double mass() const;
    [[nodiscard]] double radius() const;
    [[nodiscard]] double maxVelocity() const;
    [[nodiscard]] double maxAngularVelocity() const;
    [[nodiscard]] double centerDist() const;
    [[nodiscard]] double surfaceDist() const;
    void set_x(double value);
    void set_y(double value);
    void set_z(double value);
    void set_mass(double value);

private:
    friend class Ballpark;
    Ball(std::weak_ptr<detail::RuntimeState> state, std::int64_t id) noexcept
        : state_(std::move(state)), id_(id) {}

    [[nodiscard]] std::shared_ptr<detail::RuntimeState> state() const {
        auto state = state_.lock();
        if (!state) throw dbc::Error("Destiny Ballpark has been destroyed");
        return state;
    }
    [[nodiscard]] double get_number(const char* property) const;
    void set_number(const char* property, double value);
    std::weak_ptr<detail::RuntimeState> state_;
    std::int64_t id_;
};

class Ballpark final {
public:
    explicit Ballpark(bool is_master = false) : state_(std::make_shared<detail::RuntimeState>()) {
        call("destiny.Ballpark.__init__", "construct", "null", is_master ? "[true]" : "[false]");
    }

    Ballpark(const Ballpark&) = delete;
    Ballpark& operator=(const Ballpark&) = delete;
    Ballpark(Ballpark&&) = delete;
    Ballpark& operator=(Ballpark&&) = delete;

    [[nodiscard]] Ball AddBall(
        std::int64_t source_id, double mass, double radius, double max_velocity,
        bool is_free, bool is_global, bool is_massive, bool is_interactive,
        bool is_space_junk, double x, double y, double z, double vx, double vy,
        double vz, double agility, double speed_fraction
    ) {
        std::ostringstream args;
        args.imbue(std::locale::classic());
        args << '[' << source_id << ',' << detail::number(mass) << ',' << detail::number(radius)
             << ',' << detail::number(max_velocity) << ',' << detail::boolean(is_free)
             << ',' << detail::boolean(is_global) << ',' << detail::boolean(is_massive)
             << ',' << detail::boolean(is_interactive) << ',' << detail::boolean(is_space_junk)
             << ',' << detail::number(x) << ',' << detail::number(y) << ',' << detail::number(z)
             << ',' << detail::number(vx) << ',' << detail::number(vy) << ',' << detail::number(vz)
             << ',' << detail::number(agility) << ',' << detail::number(speed_fraction) << ']';
        call("destiny.Ballpark.AddBall", "call", "\"park:0\"", args.str());
        return Ball(state_, source_id);
    }

    void SetBallPosition(std::int64_t id, double x, double y, double z) {
        call_park("destiny.Ballpark.SetBallPosition", id, x, y, z);
    }
    void SetBallVelocity(std::int64_t id, double x, double y, double z) {
        call_park("destiny.Ballpark.SetBallVelocity", id, x, y, z);
    }
    void SetBallAngularVelocity(std::int64_t id, double x, double y, double z) {
        call_park("destiny.Ballpark.SetBallAngularVelocity", id, x, y, z);
    }
    void SetBallRotation(std::int64_t id, double x, double y, double z, double w) {
        const std::string args = "[" + std::to_string(id) + "," + detail::number(x) + "," +
            detail::number(y) + "," + detail::number(z) + "," + detail::number(w) + "]";
        call("destiny.Ballpark.SetBallRotation", "call", "\"park:0\"", args);
    }
    void SetBallMass(std::int64_t id, double value) { call_park_scalar("destiny.Ballpark.SetBallMass", id, value); }
    void SetMaxSpeed(std::int64_t id, double value) { call_park_scalar("destiny.Ballpark.SetMaxSpeed", id, value); }
    void SetMaxAngularSpeed(std::int64_t id, double value) { call_park_scalar("destiny.Ballpark.SetMaxAngularSpeed", id, value); }
    void RemoveBall(std::int64_t id, std::int64_t delay = 0) {
        call("destiny.Ballpark.RemoveBall", "call", "\"park:0\"",
            "[" + std::to_string(id) + "," + std::to_string(delay) + "]");
    }
    void ClearAll() { call("destiny.Ballpark.ClearAll", "call", "\"park:0\"", "[]"); }
    void Pause() { call("destiny.Ballpark.Pause", "call", "\"park:0\"", "[]"); }
    void Start() { call("destiny.Ballpark.Start", "call", "\"park:0\"", "[]"); }
    /** Advance one host-driven update; it steps only while Start() is active. */
    void Update() { (void)state_->update(); }
    void Evolve() { call("destiny.Ballpark.Evolve", "call", "\"park:0\"", "[]"); }
    void AdjustTimes(std::int64_t delta) {
        call("destiny.Ballpark.AdjustTimes", "call", "\"park:0\"", "[" + std::to_string(delta) + "]");
    }
    [[nodiscard]] bool isRunning() {
        return detail::exact_boolean(call("destiny.Ballpark.isRunning", "get", "\"park:0\"", "[]"));
    }
    [[nodiscard]] std::optional<double> GetSurfaceDist(std::int64_t first, std::int64_t second) {
        return detail::optional_double(call("destiny.Ballpark.GetSurfaceDist", "call", "\"park:0\"",
            "[" + std::to_string(first) + "," + std::to_string(second) + "]"));
    }
    [[nodiscard]] std::optional<double> GetCenterDist(std::int64_t first, std::int64_t second) {
        return detail::optional_double(call("destiny.Ballpark.GetCenterDist", "call", "\"park:0\"",
            "[" + std::to_string(first) + "," + std::to_string(second) + "]"));
    }
    [[nodiscard]] std::int64_t currentTime() {
        return detail::exact_integer(call("destiny.Ballpark.currentTime", "get", "\"park:0\"", "[]"));
    }
    [[nodiscard]] std::int64_t time() {
        return detail::exact_integer(call("destiny.Ballpark.time", "get", "\"park:0\"", "[]"));
    }

private:
    friend class Ball;
    std::string call(std::string_view title, std::string_view operation,
                     std::string_view target_json, std::string_view args_json) {
        return state_->call(title, operation, target_json, args_json);
    }
    void call_park(const char* title, std::int64_t id, double x, double y, double z) {
        call(title, "call", "\"park:0\"", "[" + std::to_string(id) + "," + detail::number(x) +
            "," + detail::number(y) + "," + detail::number(z) + "]");
    }
    void call_park_scalar(const char* title, std::int64_t id, double value) {
        call(title, "call", "\"park:0\"", "[" + std::to_string(id) + "," + detail::number(value) + "]");
    }
    std::shared_ptr<detail::RuntimeState> state_;
};

inline double Ball::get_number(const char* property) const {
    const std::string title = std::string("destiny.Ball.") + property;
    return detail::exact_double(state()->call(title, "get", "\"ball:" + std::to_string(id_) + "\"", "[]"));
}
inline void Ball::set_number(const char* property, double value) {
    const std::string title = std::string("destiny.Ball.") + property;
    state()->call(title, "set", "\"ball:" + std::to_string(id_) + "\"", "[" + detail::number(value) + "]");
}
inline double Ball::x() const { return get_number("x"); }
inline double Ball::y() const { return get_number("y"); }
inline double Ball::z() const { return get_number("z"); }
inline double Ball::mass() const { return get_number("mass"); }
inline double Ball::radius() const { return get_number("radius"); }
inline double Ball::maxVelocity() const { return get_number("maxVelocity"); }
inline double Ball::maxAngularVelocity() const { return get_number("maxAngularVelocity"); }
inline double Ball::centerDist() const {
    return detail::exact_double(state()->call("destiny.ClientBall.centerDist", "get",
        "\"ball:" + std::to_string(id_) + "\"", "[]"));
}
inline double Ball::surfaceDist() const {
    return detail::exact_double(state()->call("destiny.ClientBall.surfaceDist", "get",
        "\"ball:" + std::to_string(id_) + "\"", "[]"));
}
inline void Ball::set_x(double value) { set_number("x", value); }
inline void Ball::set_y(double value) { set_number("y", value); }
inline void Ball::set_z(double value) { set_number("z", value); }
inline void Ball::set_mass(double value) { set_number("mass", value); }

} // namespace destiny

#endif // DESTINY_CARBON_HPP
