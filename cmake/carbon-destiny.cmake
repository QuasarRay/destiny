# Carbon-compatible target entry point retained for consumers that include the
# exported target file directly instead of using find_package.
include_guard(GLOBAL)
if(NOT EXISTS "${CMAKE_CURRENT_LIST_DIR}/carbon-destinyConfig.cmake")
    message(FATAL_ERROR "carbon-destiny.cmake must be used from an installed or staged package")
endif()
include("${CMAKE_CURRENT_LIST_DIR}/carbon-destinyConfig.cmake")
