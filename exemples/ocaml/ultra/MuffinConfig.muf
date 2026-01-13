!muf 4
;; MuffinConfig.muf — OCaml Ultra Project (Astronomy Logbook)
;; Usage:
;;   muffin run --root . --file MuffinConfig.muf --bake app
;;   muffin run --root . --file MuffinConfig.muf --bake tests

[workspace]
  .set name "ocaml-ultra"
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

[tool ocamlopt]
  .exec "ocamlopt"
..

[tool ocamlc]
  .exec "ocamlc"
..

[bake app]
  .make core_src cglob "src/core/*.ml"
  .make app_src cglob "src/app/*.ml"
  [run ocamlopt]
    .takes core_src as "@args"
    .set "-I" "src/core"
    .set "-I" "src/app"
    .set "-I" "+unix"
    .set "unix.cmxa" 1
    .set "-O${opt}" 1
    .set "-g" "${debug}"
    .set "-w" "A-4-6-7-9-27-29"
    .emits exe as "-o"
  ..
  .output exe "target/bin/ocaml_ultra"
..

[bake tests]
  .make core_src cglob "src/core/*.ml"
  .make app_src cglob "src/app/*.ml"
  .make test_src cglob "tests/**/*.ml"
  [run ocamlc]
    .takes core_src as "@args"
    .set "-I" "src/core"
    .set "-I" "src/app"
    .set "-I" "tests"
    .set "-I" "+unix"
    .set "unix.cma" 1
    .set "-g" "${debug}"
    .emits exe as "-o"
  ..
  .output exe "target/bin/ocaml_ultra_tests.byte"
..

[export]
  .ref app
  .ref tests
..
