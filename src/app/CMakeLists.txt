# Native shell only in this slice: no core wiring, no platform adapters yet.
# The composition root (and the core links) arrive with the vertical slice that
# actually needs them.
add_executable(cliplingo WIN32 main.cpp)
target_compile_features(cliplingo PRIVATE cxx_std_20)
target_compile_definitions(cliplingo PRIVATE UNICODE _UNICODE WIN32_LEAN_AND_MEAN NOMINMAX)
target_link_libraries(cliplingo PRIVATE user32 gdi32)

if(MSVC)
  target_compile_options(cliplingo PRIVATE /W4 /permissive- /EHsc /utf-8)
else()
  target_compile_options(cliplingo PRIVATE -Wall -Wextra -Wpedantic)
endif()
