cmake_minimum_required(VERSION 3.21)
project(ClipLingo VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(NOT WIN32)
  message(STATUS "ClipLingo is Windows-only; no targets configured on this host.")
  return()
endif()

if(MSVC)
  add_compile_options(/W4 /permissive- /utf-8)
else()
  add_compile_options(-Wall -Wextra -Wpedantic)
endif()

add_subdirectory(src/contracts)
add_subdirectory(src/core/translation)
add_subdirectory(src/core/preferences)
add_subdirectory(src/core/session)
add_subdirectory(src/app)
