!muf 4
;; MuffinConfig.muf — GCC/C++ Ultra Project (Telemetry Aggregator)
;; Usage:
;;   muffin run --root . --file MuffinConfig.muf --bake app_c
;;   muffin run --root . --file MuffinConfig.muf --bake app_cpp
;;   muffin run --root . --file MuffinConfig.muf --bake tests

[workspace]
  .set name "gcc-ultra"
  .set root "."
  .set target_dir "target"
  .set profile "release"
..

[profile debug]
  .set opt 0
  .set debug 1
  .set ndebug 0
..

[profile release]
  .set opt 2
  .set debug 0
  .set ndebug 1
..

[tool gcc]
  .exec "gcc"
..

[tool gpp]
  .exec "g++"
..

[bake app_c]
  .make c_src cglob "src/c/*.c"
  [run gcc]
    .takes c_src as "@args"
    .set "-std=c17" 1
    .set "-O${opt}" 1
    .set "-g" "${debug}"
    .set "-DNDEBUG" "${ndebug}"
    .set "-Wall" 1
    .set "-Wextra" 1
    .emits exe as "-o"
  ..
  .output exe "target/out/telemetry_c"
..

[bake app_cpp]
  .make cpp_src cglob "src/cpp/*.cpp"
  [run gpp]
    .takes cpp_src as "@args"
    .set "-std=c++20" 1
    .set "-O${opt}" 1
    .set "-g" "${debug}"
    .set "-DNDEBUG" "${ndebug}"
    .set "-Wall" 1
    .set "-Wextra" 1
    .emits exe as "-o"
  ..
  .output exe "target/out/telemetry_cpp"
..

[bake tests]
  .make test_src cglob "tests/*.c"
  [run gcc]
    .takes test_src as "@args"
    .set "-std=c17" 1
    .set "-O${opt}" 1
    .set "-g" "${debug}"
    .set "-Wall" 1
    .set "-Wextra" 1
    .emits exe as "-o"
  ..
  .output exe "target/out/telemetry_tests"
..

[export]
  .ref app_c
  .ref app_cpp
  .ref tests
..
