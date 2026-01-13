!muf 4
;; MuffinConfig_multi.muf — multi-bakes + deps (gcc)
;; Usage:
;;   muffin run --root examples/gcc --file MuffinConfig_multi.muf --all

[workspace]
  .set name "app"
  .set target_dir "target"
  ;; switch: "debug" or "release"
  .set profile "debug"
  ;; shared gcc args
  .set inc_dir "lib"
  .set lib_dir "target/out"
  .set lib_name "mylib"
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

;; tool for static archive
[tool ar]
  .exec "ar"
..

;; Bake: lib (compiled first)
[bake lib]
  .make lib_src cglob "lib/**/*.c"

  [run gcc]
    .takes lib_src as "@args"
    .set "-std=c17" 1
    .set "-c" 1
    .set "-O${opt}" 1
    .set "-g" "${debug}"
    .set "-DNDEBUG" "${ndebug}"
    .set "-Wall" 1
    .set "-Wextra" 1
    .set "-o" "target/out/lib.o"
  ..

  [run ar]
    .set "rcs" 1
    .set "target/out/libmylib.a" 1
    .set "target/out/lib.o" 1
  ..

  .output liba "target/out/libmylib.a"
..

;; Bake: app (depends on lib)
[bake app]
  .needs lib
  .make app_src cglob "app/**/*.c"

  [run gcc]
    .takes app_src as "@args"
    .set "-std=c17" 1
    .set "-O${opt}" 1
    .set "-g" "${debug}"
    .set "-DNDEBUG" "${ndebug}"
    .set "-Wall" 1
    .set "-Wextra" 1
    .include "${inc_dir}"
    .libdir "${lib_dir}"
    .lib "${lib_name}"
    .emits exe as "-o"
  ..

  .output exe "target/out/app"
..

[export]
  .ref app
..
