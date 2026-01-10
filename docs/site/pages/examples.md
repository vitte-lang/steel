---
title: Examples
slug: /examples/
description: Index des exemples et commandes associees.
---

# Examples

## GCC (C)

Chemin : `examples/gcc`

Commandes :

```sh
muffin run --root examples/gcc --file MuffinConfig.muf --bake app
muffin run --root examples/gcc --file MuffinConfig_multi.muf --bake app
muffin run --root examples/gcc --file MuffinConfig_multi.muf --all
```

Outputs :

- `examples/gcc/target/out/app`
- `examples/gcc/target/out/libmylib.a`
