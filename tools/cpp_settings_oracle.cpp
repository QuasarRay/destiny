// Differential oracle compiling the ACTUAL original src/Settings.cpp.
// CCP Blue runtime declarations are supplied at the external boundary only.
// The executable C++ implementations of GlobalSettings methods are NOT copied.
//
// Settings.cpp includes Settings.h; its guard is pre-defined to substitute
// an isolated miniature Blue/Python ABI. Header names, field order, defaults
// and signatures are separately checked against the real Settings.h in CI.
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <iostream>
#include <string>

#define DESTINY_SETTINGS_H
using std::size_t;
constexpr size_t g_collisionMaxIterationsDefault = 20;
constexpr bool g_useIterativeCollisionDefault = false;
constexpr bool g_useDynamicalOrientationDefault = false;
constexpr bool g_disableDynamicalOrientationForMissilesDefault = false;
constexpr bool g_useNewOrbitDefault = false;

struct PyObject {
    virtual ~PyObject() = default;
};
struct SettingsConfiguration : PyObject {
    size_t m_collisionMaxIterations = g_collisionMaxIterationsDefault;
    bool m_useIterativeCollision = g_useIterativeCollisionDefault;
    bool m_useDynamicalOrientation = g_useDynamicalOrientationDefault;
    bool m_disableDynamicalOrientationForMissiles =
        g_disableDynamicalOrientationForMissilesDefault;
    bool m_useNewOrbit = g_useNewOrbitDefault;
};
struct GlobalSettings : PyObject {
    PyObject* ApplyConfiguration(PyObject*);
    PyObject* GetConfiguration(PyObject*);
    PyObject* GetDefaultConfiguration(PyObject*);
    PyObject* ResetConfiguration(PyObject*);
};
struct OGlobalSettings : GlobalSettings {};
struct OSettingsConfiguration : SettingsConfiguration {};
struct SettingsConfigurationPtr {
    SettingsConfiguration* ptr;
    SettingsConfigurationPtr(SettingsConfiguration* p) : ptr(p) {}
    SettingsConfigurationPtr(PyObject* p) :
        ptr(dynamic_cast<SettingsConfiguration*>(p)) {}
    explicit operator bool() const { return ptr != nullptr; }
    SettingsConfiguration* operator->() const { return ptr; }
};
struct ArgTuple : PyObject {
    PyObject* first;
    explicit ArgTuple(PyObject* value) : first(value) {}
};
static PyObject py_none;
static PyObject py_type_error;
static std::string last_error;
static PyObject* PyExc_TypeError = &py_type_error;
#define Py_RETURN_NONE return &py_none
inline int PyArg_ParseTuple(PyObject* arguments, const char* format, PyObject** out) {
    if (std::strcmp(format, "O") != 0) { return 0; }
    auto* args = dynamic_cast<ArgTuple*>(arguments);
    if (!args) { return 0; }
    *out = args->first;
    return 1;
}
inline void PyErr_SetString(PyObject*, const char* message) {
    last_error = message;
}
inline PyObject* BlueWrapObjectForPython(SettingsConfigurationPtr config) {
    return config.ptr;
}

#include "src/Settings.cpp"

void output(const char* label, SettingsConfiguration c) {
    std::cout << label << " "
              << c.m_collisionMaxIterations << " "
              << c.m_useIterativeCollision << " "
              << c.m_useDynamicalOrientation << " "
              << c.m_disableDynamicalOrientationForMissiles << " "
              << c.m_useNewOrbit << "\n";
}
SettingsConfiguration globals() {
    SettingsConfiguration c;
    c.m_collisionMaxIterations = g_collisionMaxIterations;
    c.m_useIterativeCollision = g_useIterativeCollision;
    c.m_useDynamicalOrientation = g_useDynamicalOrientation;
    c.m_disableDynamicalOrientationForMissiles =
        g_disableDynamicalOrientationForMissiles;
    c.m_useNewOrbit = g_useNewOrbit;
    return c;
}
int main() {
    output("initial", globals());
    SettingsConfiguration desired;
    desired.m_collisionMaxIterations = 5;
    desired.m_useIterativeCollision = true;
    desired.m_useDynamicalOrientation = true;
    desired.m_disableDynamicalOrientationForMissiles = true;
    desired.m_useNewOrbit = true;
    ArgTuple apply_args(&desired);
    if (s_globalSettings->ApplyConfiguration(&apply_args) != &py_none) { return 2; }
    output("immediate", globals());

    PyObject* first = s_globalSettings->GetConfiguration(nullptr);
    PyObject* second = s_globalSettings->GetConfiguration(nullptr);
    if (first == second || first == &desired) { return 3; }
    output("get", *static_cast<SettingsConfiguration*>(first));
    auto* first_cfg = static_cast<SettingsConfiguration*>(first);
    first_cfg->m_collisionMaxIterations = 999;
    output("snapshot_independence", globals());

    PyObject* defaults = s_globalSettings->GetDefaultConfiguration(nullptr);
    output("get_default", *static_cast<SettingsConfiguration*>(defaults));

    ArgTuple wrong(&py_none);
    if (s_globalSettings->ApplyConfiguration(&wrong) != nullptr) { return 4; }
    if (last_error != "Argument 1 is not a SettingsConfiguration") { return 5; }
    output("invalid_argument_unchanged", globals());

    if (s_globalSettings->ResetConfiguration(nullptr) != &py_none) { return 6; }
    output("reset", globals());
    return 0;
}
