cmake_minimum_required(VERSION 3.25)
project(ClipLingo VERSION 0.1.0 LANGUAGES CXX)
if(NOT WIN32)
  message(FATAL_ERROR "ClipLingo M1 requires Windows 11 x64 and the MSVC toolchain")
endif()
if(NOT MSVC)
  message(FATAL_ERROR "M1 validated toolchain target is MSVC on Windows")
endif()
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
option(CLIPLINGO_HOST_SMOKE "Build a Windows API compile-only target with no inference dependency" OFF)
if(CLIPLINGO_HOST_SMOKE)
  set(CLIPLINGO_TARGET ClipLingoHostSmoke)
  set(CLIPLINGO_TRANSLATION_SOURCE tests/translation_smoke.cpp)
else()
  set(CLIPLINGO_TARGET ClipLingo)
  set(CLIPLINGO_TRANSLATION_SOURCE src/translation.cpp)
  find_package(ctranslate2 CONFIG REQUIRED)
  # Prefer the official/exported CMake target (e.g. vcpkg). Older native
  # SentencePiece source installs may only provide a header and a library.
  find_package(sentencepiece CONFIG QUIET)
  if(NOT TARGET sentencepiece::sentencepiece)
    find_path(CLIPLINGO_SENTENCEPIECE_INCLUDE sentencepiece_processor.h REQUIRED)
    find_library(CLIPLINGO_SENTENCEPIECE_LIB NAMES sentencepiece REQUIRED)
    add_library(sentencepiece::sentencepiece UNKNOWN IMPORTED)
    set_target_properties(sentencepiece::sentencepiece PROPERTIES
      IMPORTED_LOCATION "${CLIPLINGO_SENTENCEPIECE_LIB}"
      INTERFACE_INCLUDE_DIRECTORIES "${CLIPLINGO_SENTENCEPIECE_INCLUDE}")
  endif()
endif()
add_executable(${CLIPLINGO_TARGET} WIN32
  src/main.cpp
  src/capture.cpp
  src/capture_policy.cpp
  src/hover.cpp
  src/capture_helper.cpp
  src/model_package.cpp
  src/popup.cpp
  ${CLIPLINGO_TRANSLATION_SOURCE}
)
target_include_directories(${CLIPLINGO_TARGET} PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/src)
target_compile_definitions(${CLIPLINGO_TARGET} PRIVATE UNICODE _UNICODE WIN32_LEAN_AND_MEAN NOMINMAX)
target_compile_options(${CLIPLINGO_TARGET} PRIVATE /W4 /permissive- /EHsc /utf-8 /sdl)
target_link_libraries(${CLIPLINGO_TARGET} PRIVATE
  ole32 oleaut32 uiautomationcore user32 shell32 shlwapi comctl32 d2d1 dwrite dwmapi shcore bcrypt)
if(NOT CLIPLINGO_HOST_SMOKE)
  target_link_libraries(${CLIPLINGO_TARGET} PRIVATE CTranslate2::ctranslate2 sentencepiece::sentencepiece)
endif()
set_property(TARGET ${CLIPLINGO_TARGET} PROPERTY VS_DPI_AWARE "PerMonitor")

if(CLIPLINGO_HOST_SMOKE)
  add_executable(ClipLingoModelPackTest tests/model_pack_test.cpp src/model_package.cpp)
  target_include_directories(ClipLingoModelPackTest PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/src)
  target_compile_definitions(ClipLingoModelPackTest PRIVATE UNICODE _UNICODE WIN32_LEAN_AND_MEAN NOMINMAX)
  target_compile_options(ClipLingoModelPackTest PRIVATE /W4 /permissive- /EHsc /utf-8 /sdl)
  target_link_libraries(ClipLingoModelPackTest PRIVATE bcrypt)
  add_executable(ClipLingoScopeTextTest tests/scope_text_test.cpp)
  target_include_directories(ClipLingoScopeTextTest PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/src)
  target_compile_options(ClipLingoScopeTextTest PRIVATE /W4 /permissive- /EHsc /utf-8)
  enable_testing()
  add_test(NAME model_pack_integrity COMMAND ClipLingoModelPackTest)
  add_test(NAME scope_text COMMAND ClipLingoScopeTextTest)
  add_executable(ClipLingoCapturePolicyTest tests/capture_policy_test.cpp)
  target_include_directories(ClipLingoCapturePolicyTest PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/src)
  target_compile_options(ClipLingoCapturePolicyTest PRIVATE /W4 /WX /permissive- /EHsc /utf-8)
  add_test(NAME capture_policy COMMAND ClipLingoCapturePolicyTest)
endif()