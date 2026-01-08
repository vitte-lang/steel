# Targets

Référence (index) des **targets** dans Muffin.

Une *target* décrit l’environnement de compilation/exécution visé (OS/arch/ABI), et pilote :

- la résolution des extensions (obj/lib/exe)
- la sélection des toolchains (si déclarées)
- les flags de compilation (selon impl)
- les contraintes sandbox (capsule) et I/O (selon OS)

---

## Rappels

- Buildfiles `*.muf/*.muff` décrivent le graph.
- `muffin configure` compile le graph en `Muffinconfig.mff` en résolvant `target`.
- `muffin build` exécute le DAG en respectant les choix de `target`.

---

## Modèle

### Triple

Recommandation : utiliser un **triple** de type LLVM/Rust :

```text
<arch>-<vendor>-<os>-<abi>
```

Exemples :

- `x86_64-unknown-linux-gnu`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `x86_64-unknown-freebsd`
- `x86_64-unknown-solaris`

### Target spec (concept)

Une target peut être décrite par :

- `triple`
- `os` (dérivé)
- `arch` (dérivé)
- `endianness` (optionnel)
- `pointer_width` (optionnel)
- `libc/abi` (optionnel)

---

## Résolution des sorties (extensions)

Muffin manipule des **types logiques** et résout l’extension finale selon `target`.

### Objets

- Unix : `.o`
- Windows : `.obj`

### Librairies statiques

- Unix : `.a`
- Windows : `.lib`

### Librairies partagées

- Linux/BSD/Solaris : `.so`
- macOS : `.dylib`
- Windows : `.dll`

### Exécutables

- Unix : (sans extension)
- Windows : `.exe`

---

## Sources de configuration

### 1) Buildfile

- variables (`var target: text = ...`)
- profile (`profile release set target ...`)
- switch (mapping flags)

### 2) CLI

- `--target <triple>`
- `-D target=<triple>`

### 3) Environnement

- `MUFFIN_TARGET=<triple>` (pattern)

Priorité recommandée : CLI > env > buildfile.

---

## Usage

### Configure

```text
muffin configure --target x86_64-unknown-linux-gnu
muffin configure -D target=aarch64-apple-darwin
```

### Build

Le build exécute le target déjà résolu dans `.mff`.

```text
muffin build --mff Muffinconfig.mff
```

---

## Multi-target

### Pattern (concept)

- exécuter `configure` par target
- produire un `.mff` distinct par target

Exemple :

```text
muffin configure --target x86_64-unknown-linux-gnu --out Muffinconfig.linux.mff
muffin configure --target x86_64-pc-windows-msvc --out Muffinconfig.windows.mff
muffin configure --target aarch64-apple-darwin --out Muffinconfig.macos.mff
```

---

## Cross-compilation

Recommandations :

- toolchains déclarées par target
- pinning version/checksum
- capsules adaptées (FS/env minimal)
- éviter les chemins absolus dans les outputs

---

## Validation

Règles recommandées :

- triple parseable
- mapping extensions cohérent
- toolchain compatible
- outputs réalisables sur l’OS hôte (sinon cross toolchain)

Commandes utiles :

```text
muffin doctor --tools
muffin decompile Muffinconfig.mff --format json
```

---

## Notes OS

- Linux : globs case-sensitive en général
- macOS : `.dylib`, `shasum -a 256`
- Windows : `.exe/.dll/.lib/.obj`, PowerShell `Get-FileHash`
- BSD : similar Linux
- Solaris : similar Linux (toolchains variables)

---

## Voir aussi

- CLI : `docs/reference/cli/index.md`
- Config : `docs/reference/config/index.md`
- Formats : `docs/reference/formats/index.md`
- Ops (releases/signing) : `docs/ops/index.md`
