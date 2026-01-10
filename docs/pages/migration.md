---
title: Migration MUF
slug: /migration/
description: Passer a MUF v4.1 depuis les anciens formats.
---

# Migration MUF v4.1

## Header

Ancien:

```text
muf 2
```

Nouveau:

```text
!muf 4
```

## Blocs

Ancien:

```text
workspace
  set name "app"
.end
```

Nouveau:

```text
[workspace]
  .set name "app"
..
```

## Directives

Ancien:

```text
set name "app"
make c_src cglob "**/*.c"
```

Nouveau:

```text
.set name "app"
.make c_src cglob "**/*.c"
```

## Commentaires

```text
;; commentaire MUF v4.1
```

## Validation

```sh
muffin run --root . --file MuffinConfig.muf --print
```
