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
find_package(ctranslate2 CONFIG REQUIRED)
find_package(sentencepiece CONFIG REQUIRED)
add_executable(ClipLingo WIN32
  src/main.cpp
  src/capture.cpp
  src/popup.cpp
  src/translation.cpp
)
target_compile_definitions(ClipLingo PRIVATE UNICODE _UNICODE WIN32_LEAN_AND_MEAN NOMINMAX)
target_compile_options(ClipLingo PRIVATE /W4 /permissive- /EHsc /utf-8 /sdl)
target_link_libraries(ClipLingo PRIVATE
  CTranslate2::ctranslate2 sentencepiece::sentencepiece
  ole32 oleaut32 uiautomationcore user32 shell32 shlwapi comctl32 d2d1 dwrite dwmapi shcore)
set_property(TARGET ClipLingo PROPERTY VS_DPI_AWARE "PerMonitor")