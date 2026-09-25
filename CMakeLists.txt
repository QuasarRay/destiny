cmake_minimum_required(VERSION 3.24)
project(destiny_bevy_compat VERSION 0.4.0 LANGUAGES C CXX)

include(GNUInstallDirs)
include(CMakePackageConfigHelpers)
include(CTest)

option(DBC_ENABLE_CARBON_NETWORK "Compile Lightyear and Replicon interoperability" ON)
option(DBC_CARGO_RELEASE "Build the Rust runtime in release mode" ON)
option(DBC_CARGO_LOCKED "Require Cargo.lock during native builds" ON)
set(DBC_CARGO_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/cargo" CACHE PATH "Cargo target directory")

find_program(DBC_CARGO_EXECUTABLE cargo REQUIRED)
if(DBC_CARGO_LOCKED AND NOT EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.lock")
    message(FATAL_ERROR "DBC_CARGO_LOCKED=ON requires the committed Cargo.lock")
endif()

if(DBC_CARGO_RELEASE)
    set(DBC_CARGO_PROFILE release)
    set(DBC_CARGO_PROFILE_FLAG --release)
else()
    set(DBC_CARGO_PROFILE debug)
    set(DBC_CARGO_PROFILE_FLAG)
endif()

set(DBC_CARGO_FEATURE_ARGS)
if(NOT DBC_ENABLE_CARBON_NETWORK)
    list(APPEND DBC_CARGO_FEATURE_ARGS --no-default-features)
endif()
set(DBC_CARGO_LOCK_ARGS)
if(DBC_CARGO_LOCKED)
    list(APPEND DBC_CARGO_LOCK_ARGS --locked)
endif()

if(WIN32)
    set(DBC_RUNTIME_FILENAME destiny_bevy_compat.dll)
    set(DBC_LINK_FILENAME destiny_bevy_compat.dll.lib)
elseif(APPLE)
    set(DBC_RUNTIME_FILENAME libdestiny_bevy_compat.dylib)
    set(DBC_LINK_FILENAME ${DBC_RUNTIME_FILENAME})
else()
    set(DBC_RUNTIME_FILENAME libdestiny_bevy_compat.so)
    set(DBC_LINK_FILENAME ${DBC_RUNTIME_FILENAME})
endif()

set(DBC_RUNTIME_PATH "${DBC_CARGO_TARGET_DIR}/${DBC_CARGO_PROFILE}/${DBC_RUNTIME_FILENAME}")
set(DBC_LINK_PATH "${DBC_CARGO_TARGET_DIR}/${DBC_CARGO_PROFILE}/${DBC_LINK_FILENAME}")

file(GLOB_RECURSE DBC_RUST_SOURCES CONFIGURE_DEPENDS
    "${CMAKE_CURRENT_SOURCE_DIR}/src/*.rs"
)
set(DBC_CARGO_DEPENDS
    Cargo.toml
    rust-toolchain.toml
    ${DBC_RUST_SOURCES}
)
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.lock")
    list(APPEND DBC_CARGO_DEPENDS Cargo.lock)
endif()

set(DBC_CARGO_OUTPUTS "${DBC_RUNTIME_PATH}")
if(WIN32)
    list(APPEND DBC_CARGO_OUTPUTS "${DBC_LINK_PATH}")
endif()

add_custom_command(
    OUTPUT ${DBC_CARGO_OUTPUTS}
    COMMAND "${DBC_CARGO_EXECUTABLE}" build
            --manifest-path "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml"
            --target-dir "${DBC_CARGO_TARGET_DIR}"
            ${DBC_CARGO_PROFILE_FLAG}
            ${DBC_CARGO_FEATURE_ARGS}
            ${DBC_CARGO_LOCK_ARGS}
    DEPENDS ${DBC_CARGO_DEPENDS}
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    COMMENT "Building Destiny-to-Bevy Rust runtime"
    VERBATIM
)

add_custom_target(destiny_bevy_compat_rust ALL DEPENDS "${DBC_RUNTIME_PATH}")

add_library(destiny_bevy_compat SHARED IMPORTED GLOBAL)
set_target_properties(destiny_bevy_compat PROPERTIES
    IMPORTED_LOCATION "${DBC_RUNTIME_PATH}"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_SOURCE_DIR}/include"
)
if(WIN32)
    set_target_properties(destiny_bevy_compat PROPERTIES IMPORTED_IMPLIB "${DBC_LINK_PATH}")
endif()
add_dependencies(destiny_bevy_compat destiny_bevy_compat_rust)

add_library(destiny::bevy_compat ALIAS destiny_bevy_compat)
add_library(Destiny INTERFACE)
target_link_libraries(Destiny INTERFACE destiny_bevy_compat)
add_library(destiny ALIAS Destiny)
add_library(carbon-destiny::destiny ALIAS Destiny)

if(BUILD_TESTING)
    add_executable(dbc_c_abi_smoke tests/native/c_abi_smoke.c)
    target_link_libraries(dbc_c_abi_smoke PRIVATE destiny::bevy_compat)
    add_test(NAME dbc_c_abi_smoke COMMAND dbc_c_abi_smoke)

    add_executable(dbc_cpp_facade_smoke tests/native/cpp_facade_smoke.cpp)
    target_compile_features(dbc_cpp_facade_smoke PRIVATE cxx_std_17)
    target_link_libraries(dbc_cpp_facade_smoke PRIVATE destiny::bevy_compat)
    add_test(NAME dbc_cpp_facade_smoke COMMAND dbc_cpp_facade_smoke)

    if(WIN32)
        set_tests_properties(dbc_c_abi_smoke dbc_cpp_facade_smoke PROPERTIES
            ENVIRONMENT_MODIFICATION "PATH=path_list_prepend:${DBC_CARGO_TARGET_DIR}/${DBC_CARGO_PROFILE}"
        )
    endif()
endif()

if(SKBUILD)
    # scikit-build-core installs into the wheel's platlib tree. The ctypes
    # loader searches beside the Python package, so the native library and
    # facade are always shipped together in a platform wheel.
    install(FILES "${DBC_RUNTIME_PATH}" DESTINATION "${SKBUILD_PLATLIB_DIR}/destiny")
    install(DIRECTORY python/destiny DESTINATION "${SKBUILD_PLATLIB_DIR}"
            FILES_MATCHING PATTERN "*.py" PATTERN "package.yaml")
    install(FILES python/_destiny.py DESTINATION "${SKBUILD_PLATLIB_DIR}")
else()
    install(FILES
        include/destiny_bevy_compat.h
        include/destiny_bevy_compat.hpp
        include/destiny_carbon.hpp
        DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}"
    )
    if(WIN32)
        install(FILES "${DBC_RUNTIME_PATH}" DESTINATION "${CMAKE_INSTALL_BINDIR}")
        install(FILES "${DBC_LINK_PATH}" DESTINATION "${CMAKE_INSTALL_LIBDIR}")
    else()
        install(FILES "${DBC_RUNTIME_PATH}" DESTINATION "${CMAKE_INSTALL_LIBDIR}")
    endif()
    install(DIRECTORY python/destiny DESTINATION bin/python FILES_MATCHING PATTERN "*.py" PATTERN "package.yaml")
    install(FILES python/_destiny.py DESTINATION bin/python)

    configure_package_config_file(
        cmake/carbon-destinyConfig.cmake.in
        "${CMAKE_CURRENT_BINARY_DIR}/carbon-destinyConfig.cmake"
        INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/carbon-destiny"
        PATH_VARS CMAKE_INSTALL_BINDIR CMAKE_INSTALL_INCLUDEDIR CMAKE_INSTALL_LIBDIR
    )
    configure_package_config_file(
        cmake/carbon-destinyConfig.cmake.in
        "${CMAKE_CURRENT_BINARY_DIR}/carbon-destinyLegacyConfig.cmake"
        INSTALL_DESTINATION "share/carbon-destiny"
        PATH_VARS CMAKE_INSTALL_BINDIR CMAKE_INSTALL_INCLUDEDIR CMAKE_INSTALL_LIBDIR
    )
    configure_package_config_file(
        cmake/carbon-destinyConfig.cmake.in
        "${CMAKE_CURRENT_BINARY_DIR}/DestinyConfig.cmake"
        INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/Destiny"
        PATH_VARS CMAKE_INSTALL_BINDIR CMAKE_INSTALL_INCLUDEDIR CMAKE_INSTALL_LIBDIR
    )
    write_basic_package_version_file(
        "${CMAKE_CURRENT_BINARY_DIR}/carbon-destinyConfigVersion.cmake"
        VERSION "${PROJECT_VERSION}"
        COMPATIBILITY SameMajorVersion
    )
    write_basic_package_version_file(
        "${CMAKE_CURRENT_BINARY_DIR}/DestinyConfigVersion.cmake"
        VERSION "${PROJECT_VERSION}"
        COMPATIBILITY SameMajorVersion
    )

    install(FILES
        cmake/carbon-destiny.cmake
        "${CMAKE_CURRENT_BINARY_DIR}/carbon-destinyConfig.cmake"
        "${CMAKE_CURRENT_BINARY_DIR}/carbon-destinyConfigVersion.cmake"
        DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/carbon-destiny"
    )
    install(FILES
        "${CMAKE_CURRENT_BINARY_DIR}/DestinyConfig.cmake"
        "${CMAKE_CURRENT_BINARY_DIR}/DestinyConfigVersion.cmake"
        DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/Destiny"
    )
    install(FILES
        cmake/carbon-destiny.cmake
        "${CMAKE_CURRENT_BINARY_DIR}/carbon-destinyConfigVersion.cmake"
        DESTINATION "share/carbon-destiny"
    )
    install(FILES
        "${CMAKE_CURRENT_BINARY_DIR}/carbon-destinyLegacyConfig.cmake"
        DESTINATION "share/carbon-destiny"
        RENAME carbon-destinyConfig.cmake
    )
    install(FILES LICENSE THIRD_PARTY_NOTICES.md
        DESTINATION "${CMAKE_INSTALL_DATADIR}/destiny-bevy-compat"
    )
endif()
