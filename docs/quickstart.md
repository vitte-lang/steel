# Quickstart

This is a minimal path to get Muffin running with the MUF v4.1 syntax.

## Install

```sh
cargo install --path . --force
```

## Build config (MUF v4.1)

Create `MuffinConfig.muf` at the repo root:

```text
!muf 4

[workspace]
  .set name "app"
  .set target_dir "target"
  .set profile "debug"
..

[profile debug]
  .set opt 0
  .set debug 1
  .set ndebug 0
..

[tool gcc]
  .exec "gcc"
..

[bake app]
  .make c_src cglob "**/*.c"
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
  .output exe "target/out/app"
..

[export]
  .ref app
..
```

## Run

```sh
muffin run --root . --file MuffinConfig.muf
```

Dry-run:

```sh
muffin run --root . --file MuffinConfig.muf --print
```

## Logs

By default a log is written to `target/muffin_run_<timestamp>.mff`.

```sh
muffin run --root . --file MuffinConfig.muf --log target/muffin_run.mff --log-mode truncate
```

## Result check (real)

After a successful run, you should see:

- output binary at `target/out/app`
- a run log like `target/muffin_run_*.mff`

Example:

```sh
ls -l target/out/app
ls -t target/muffin_run_*.mff | head -n 1
```
