cmake_minimum_required(VERSION 3.20)

project(cliplingo_worker LANGUAGES CXX)

include(FetchContent)

# Native translation dependencies must agree on one language standard during
# their own configure checks, not only after our executable targets exist.
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# CTranslate2's static Windows build uses the static MSVC runtime. Keep all
# worker-owned targets and SentencePiece on the same runtime to avoid mixing
# /MT dependencies with /MD executables at link time.
if(MSVC)
  set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")

  # SentencePiece configures its bundled Abseil during FetchContent. On hosted
  # MSVC runners Abseil's configure-time probe does not reliably inherit the
  # target-level standard, so make the toolchain flag explicit before
  # dependency configuration.
  string(APPEND CMAKE_CXX_FLAGS " /std:c++17")
endif()

# Current hosted runners use a CMake version that removed compatibility with
# policy versions older than 3.5. CTranslate2's pinned cpu_features submodule
# still declares an older minimum, so keep the compatibility floor explicit
# at our dependency boundary rather than patching upstream source.
set(CMAKE_POLICY_VERSION_MINIMUM 3.5 CACHE STRING "" FORCE)

# Worker runtime dependencies are source-pinned and statically linked so the
# shipped worker does not depend on Python or machine-global ML runtimes.
set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set(BUILD_CLI OFF CACHE BOOL "" FORCE)
set(BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(WITH_MKL OFF CACHE BOOL "" FORCE)
set(WITH_DNNL OFF CACHE BOOL "" FORCE)
set(WITH_OPENBLAS OFF CACHE BOOL "" FORCE)
set(WITH_RUY ON CACHE BOOL "" FORCE)
set(WITH_CUDA OFF CACHE BOOL "" FORCE)
set(WITH_CUDNN OFF CACHE BOOL "" FORCE)
set(WITH_HIP OFF CACHE BOOL "" FORCE)
set(OPENMP_RUNTIME NONE CACHE STRING "" FORCE)

FetchContent_Declare(
  ctranslate2
  GIT_REPOSITORY https://github.com/OpenNMT/CTranslate2.git
  GIT_TAG d44d2d069eb88c7b7804da864c10c201501cb4a9
  GIT_SUBMODULES_RECURSE TRUE
)
FetchContent_MakeAvailable(ctranslate2)

set(SPM_ENABLE_SHARED OFF CACHE BOOL "" FORCE)
set(SPM_BUILD_TEST OFF CACHE BOOL "" FORCE)
set(SPM_ENABLE_TCMALLOC OFF CACHE BOOL "" FORCE)
set(SPM_ENABLE_MSVC_MT_BUILD ON CACHE BOOL "" FORCE)

FetchContent_Declare(
  sentencepiece
  GIT_REPOSITORY https://github.com/google/sentencepiece.git
  GIT_TAG e0cce7d37b065b5140349dbe12c6bcf6192fdd78
)
FetchContent_MakeAvailable(sentencepiece)

add_executable(cliplingo_worker main.cpp)
set_target_properties(cliplingo_worker PROPERTIES OUTPUT_NAME "cliplingo-worker")
target_compile_features(cliplingo_worker PRIVATE cxx_std_17)
target_compile_definitions(cliplingo_worker PRIVATE UNICODE _UNICODE WIN32_LEAN_AND_MEAN NOMINMAX)
target_link_libraries(cliplingo_worker PRIVATE ctranslate2 sentencepiece)

add_executable(cliplingo_worker_runtime_probe runtime_probe.cpp)
set_target_properties(
  cliplingo_worker_runtime_probe
  PROPERTIES OUTPUT_NAME "cliplingo-worker-runtime-probe"
)
target_compile_features(cliplingo_worker_runtime_probe PRIVATE cxx_std_17)
target_link_libraries(cliplingo_worker_runtime_probe PRIVATE ctranslate2 sentencepiece)

if(MSVC)
  target_compile_options(cliplingo_worker PRIVATE /W4 /permissive-)
  target_compile_options(cliplingo_worker_runtime_probe PRIVATE /W4 /permissive-)
endif()
