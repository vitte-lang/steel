!muf 4
;; MuffinConfig.muf — build C (gcc) à la racine
;; Usage:
;;   muffin build
;; ou:
;;   muffin build MuffinConfig.muf


[workspace]
  .set name "app"
  .set target_dir "target"
  ;; switch: "debug" or "release"
  .set profile "debug"
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


[bake app]
  ;; sources C
  .make c_src cglob "**/*.c"

  ;; gcc: compile + link (simple)
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

  ;; sortie finale
  .output exe "target/out/app"
..


[export]
  .ref app
..
