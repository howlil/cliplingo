add_executable(cl_tests_unit
  main.cpp
  unit/session_generation_test.cpp
  unit/request_policy_test.cpp
  unit/translation_service_test.cpp)

target_link_libraries(cl_tests_unit PRIVATE
  cl_core_session
  cl_core_translation
  cl_core_preferences
  cl_contracts)

target_include_directories(cl_tests_unit PRIVATE
  ${CMAKE_SOURCE_DIR}/src
  ${CMAKE_CURRENT_SOURCE_DIR})

add_test(NAME core_unit COMMAND cl_tests_unit)

# F1/F3/F4 cannot be enforced by the target graph alone: system headers are
# always reachable, so the include boundary is a source check (engineering.md).
add_test(NAME layer_boundaries
         COMMAND powershell -NoProfile -ExecutionPolicy Bypass
                 -File ${CMAKE_SOURCE_DIR}/tools/check_boundaries.ps1)
