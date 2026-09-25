cmake_minimum_required(VERSION 3.24)
project(destiny_bevy_compat_consumer LANGUAGES CXX)

include(CTest)
find_package(carbon-destiny 0.3 CONFIG REQUIRED)

add_executable(destiny_consumer main.cpp)
target_compile_features(destiny_consumer PRIVATE cxx_std_17)
target_link_libraries(destiny_consumer PRIVATE carbon-destiny::destiny)
add_test(NAME destiny_consumer COMMAND destiny_consumer)
