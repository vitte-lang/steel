!muf 4
;; MuffinConfig.muf — OCaml cross-OS example (native + bytecode)
;; Usage:
;;   muffin run --root . --file MuffinConfig.muf --bake app_macos
;;   muffin run --root . --file MuffinConfig.muf --bake app_linux
;;   muffin run --root . --file MuffinConfig.muf --bake app_bsd
;; Notes:
;;   - This file shows *how* to produce OS-specific binaries; actual cross-compilation
;;     requires a matching OCaml toolchain for each target on your PATH.

[workspace]
  .set name "ocaml-complete"
  .set root "."
  .set target_dir "target"
  ;; switch: "debug" or "release"
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

[tool ocamlopt]
  .exec "ocamlopt"
..

[tool ocamlc]
  .exec "ocamlc"
..

[bake app_macos]
  .make ml_src cglob "src/*.ml"
  [run ocamlopt]
    .takes ml_src as "@args"
    .set "-O${opt}" 1
    .set "-g" "${debug}"
    .set "-w" "A-4-6-7-9-27-29"
    .emits exe as "-o"
  ..
  .output exe "target/bin/macos/app_ml"
..

[bake app_linux]
  .make ml_src cglob "src/*.ml"
  [run ocamlopt]
    .takes ml_src as "@args"
    .set "-O${opt}" 1
    .set "-g" "${debug}"
    .set "-w" "A-4-6-7-9-27-29"
    .emits exe as "-o"
  ..
  .output exe "target/bin/linux/app_ml"
..

[bake app_bsd]
  .make ml_src cglob "src/*.ml"
  [run ocamlopt]
    .takes ml_src as "@args"
    .set "-O${opt}" 1
    .set "-g" "${debug}"
    .set "-w" "A-4-6-7-9-27-29"
    .emits exe as "-o"
  ..
  .output exe "target/bin/bsd/app_ml"
..

[bake app_bytecode]
  .make ml_src cglob "src/*.ml"
  [run ocamlc]
    .takes ml_src as "@args"
    .set "-g" "${debug}"
    .emits exe as "-o"
  ..
  .output exe "target/bin/bytecode/app_ml.byte"
..

[export]
  .ref app_macos
  .ref app_linux
  .ref app_bsd
  .ref app_bytecode
..
