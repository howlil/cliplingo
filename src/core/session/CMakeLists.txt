add_library(cl_core_session STATIC session_controller.cpp)
# K3 orchestrates K4 and K8; it must not see platform or ui targets (F1).
target_link_libraries(cl_core_session
  PUBLIC cl_contracts
  PUBLIC cl_core_translation
  PUBLIC cl_core_preferences)
target_include_directories(cl_core_session PUBLIC ${CMAKE_SOURCE_DIR}/src)
