cmake_minimum_required(VERSION 3.20)

project(cliplingo_worker LANGUAGES CXX)

add_executable(cliplingo_worker main.cpp)
set_target_properties(cliplingo_worker PROPERTIES OUTPUT_NAME "cliplingo-worker")
target_compile_features(cliplingo_worker PRIVATE cxx_std_17)
target_compile_definitions(cliplingo_worker PRIVATE UNICODE _UNICODE WIN32_LEAN_AND_MEAN NOMINMAX)

if(MSVC)
  target_compile_options(cliplingo_worker PRIVATE /W4 /permissive-)
endif()
