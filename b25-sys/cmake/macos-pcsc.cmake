# Use the PC/SC implementation shipped in the macOS SDK.
if(NOT APPLE)
    return()
endif()

# CMake normally supplies the SDK to the compiler, but resolve it explicitly
# before FindPCSC runs so that both headers and the framework come from the
# same SDK.  Accept either an explicit SDK path or an xcrun SDK name.
if(CMAKE_OSX_SYSROOT AND IS_ABSOLUTE "${CMAKE_OSX_SYSROOT}")
    set(_pcsc_sdk_path "${CMAKE_OSX_SYSROOT}")
else()
    if(CMAKE_OSX_SYSROOT)
        set(_pcsc_sdk_name "${CMAKE_OSX_SYSROOT}")
    else()
        set(_pcsc_sdk_name "macosx")
    endif()

    execute_process(
        COMMAND xcrun --sdk "${_pcsc_sdk_name}" --show-sdk-path
        RESULT_VARIABLE _pcsc_xcrun_result
        OUTPUT_VARIABLE _pcsc_sdk_path
        OUTPUT_STRIP_TRAILING_WHITESPACE
        ERROR_VARIABLE _pcsc_xcrun_error
    )
    if(NOT _pcsc_xcrun_result EQUAL 0 OR NOT _pcsc_sdk_path)
        message(FATAL_ERROR
            "Unable to locate the macOS SDK with xcrun: ${_pcsc_xcrun_error}"
        )
    endif()
    set(CMAKE_OSX_SYSROOT "${_pcsc_sdk_path}" CACHE PATH "macOS SDK" FORCE)
endif()

set(_pcsc_framework_path "${_pcsc_sdk_path}/System/Library/Frameworks/PCSC.framework")
set(CMAKE_FIND_FRAMEWORK FIRST)
find_path(PCSC_FRAMEWORK_INCLUDE_DIR
    NAMES winscard.h
    PATHS "${_pcsc_framework_path}/Headers"
    NO_DEFAULT_PATH
)
find_library(PCSC_FRAMEWORK_LIBRARY
    NAMES PCSC
    PATHS "${_pcsc_sdk_path}/System/Library/Frameworks"
    NO_DEFAULT_PATH
)

if(NOT PCSC_FRAMEWORK_INCLUDE_DIR OR NOT PCSC_FRAMEWORK_LIBRARY)
    message(FATAL_ERROR
        "The macOS SDK does not contain PCSC.framework (SDK: ${_pcsc_sdk_path})"
    )
endif()

# FindPCSC.cmake skips discovery when PCSC_FOUND is already set.  This keeps
# pkg-config and its libpcsclite fallback out of the macOS path.
set(PCSC_INCLUDE_DIRS "${PCSC_FRAMEWORK_INCLUDE_DIR}")
set(PCSC_LIBRARIES "${PCSC_FRAMEWORK_LIBRARY}")
set(PCSC_LIBRARY_DIRS "")
set(WITH_PCSC_PACKAGE "")
set(WITH_PCSC_LIBRARY "")
set(PCSC_FOUND TRUE)
