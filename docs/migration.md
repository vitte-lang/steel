# Migration Guide

This guide covers the shift to MUF v4.1 syntax.

## 1) Header

Old:

```text
muf 2
```

New:

```text
!muf 4
```

## 2) Blocks

Old blocks used `.end`. New syntax uses bracketed block headers and closes with `..`.

Old:

```text
workspace
  set name "app"
.end
```

New:

```text
[workspace]
  .set name "app"
..
```

## 3) Directives

Directives now use a leading dot.

Old:

```text
set name "app"
make c_src cglob "**/*.c"
```

New:

```text
.set name "app"
.make c_src cglob "**/*.c"
```

## 4) Comments

Use `;;` for comments in MUF v4.1.

```text
;; This is a comment
```

## 5) Variable expansion

Use `${var}` to expand variables.

```text
.set profile "debug"
.set "-O${opt}" 1
```

## 6) Files and names

Pick one file name and stay consistent. The current examples use:

- Config: `MuffinConfig.muf`
- Frozen config/logs: `Muffinconfig.mff`

## 7) Validate syntax

Reference grammar: `assets/grammar/muffin.ebnf`.

You can also use:

```sh
muffin run --root . --file MuffinConfig.muf --print
```

to validate the config without running tools.
