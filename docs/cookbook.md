# Cookbook

Quick recipes for MUF v4.1 using the GCC examples in this repo.

## C app (single bake)

Use the ready example:

- Config: `examples/gcc/MuffinConfig.muf`
- Source: `examples/gcc/app/main.c`

Run:

```sh
muffin run --root examples/gcc --file MuffinConfig.muf --bake app
```

Dry-run:

```sh
muffin run --root examples/gcc --file MuffinConfig.muf --bake app --print
```

Expected outputs:

- `examples/gcc/target/out/app`
- `examples/gcc/target/muffin_run_*.mff`

## Library + app (deps)

Use the multi-bake example:

- Config: `examples/gcc/MuffinConfig_multi.muf`
- Library: `examples/gcc/lib/lib.c`
- App: `examples/gcc/app/main.c`

Relevant structure:

```text
[bake lib]
  .make c_src cglob "lib/*.c"
  [run gcc]
    .takes c_src as "@args"
    .set "-c" 1
    .emits obj as "-o"
  ..
  [run ar]
    .takes obj as "@args"
    .emits liba as "@args"
  ..
  .output liba "target/out/libmylib.a"
..

[bake app]
  .needs "lib"
  .make c_src cglob "app/*.c"
  [run gcc]
    .takes c_src as "@args"
    .set "-L${lib_dir}" 1
    .set "-l${lib_name}" 1
    .emits exe as "-o"
  ..
  .output exe "target/out/app"
..
```

Run app (builds lib first):

```sh
muffin run --root examples/gcc --file MuffinConfig_multi.muf --bake app
```

Build all bakes:

```sh
muffin run --root examples/gcc --file MuffinConfig_multi.muf --all
```

Expected outputs:

- `examples/gcc/target/out/libmylib.a`
- `examples/gcc/target/out/app`

## Multi-tool (gcc + ar) with shared flags

Centralize flags in `workspace` or `profile`, then reuse them in runs:

```text
[workspace]
  .set inc_dir "lib"
  .set lib_dir "target/out"
  .set lib_name "mylib"
..

[run gcc]
  .include "${inc_dir}"
  .libdir "${lib_dir}"
  .lib "${lib_name}"
..
```

See `examples/gcc/MuffinConfig_multi.muf` for a full gcc + ar example.

## Logs

Log to a fixed file or let Muffin create a timestamped log:

```sh
muffin run --root examples/gcc --file MuffinConfig.muf --log target/muffin_run.mff --log-mode truncate
```
