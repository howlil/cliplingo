# Cross-module data contracts: values only, standard library only.
# Deliberately no user32 / winhttp / sqlite / slint here (F1). The include
# boundary is checked by tools/check_boundaries.ps1.
add_library(cl_contracts INTERFACE)
target_compile_features(cl_contracts INTERFACE cxx_std_20)
target_include_directories(cl_contracts INTERFACE ${CMAKE_SOURCE_DIR}/src)
