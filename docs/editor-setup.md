# Steelconf editor setup (terminal editors)

Steel installs these editor settings on first run. To skip it, set
`STEEL_NO_EDITOR_SETUP=1`. If a config file already exists (for example
`languages.toml` or `settings.json`), Steel will not overwrite it and will leave
an `editor-setup.todo` note under `~/.config/steel/`.

You can re-run the installer at any time:
```
steel editor-setup
```

This document shows the exact files written.

These snippets add a basic filetype and 2-space indentation for `steelconf` and
`*.muf` files. Tabs are converted to spaces.

## steecleditor config
Create `~/.config/steel/steecleditor.conf` (or `XDG_CONFIG_HOME/steel/steecleditor.conf`) to override defaults:
```
# Autosave interval in seconds (0 disables autosave)
autosave_interval=30

# Theme: dark or light
theme=dark
```
You can toggle theme in the editor with `Ctrl+M`.
You can open completion debug view with `Ctrl+Shift+L`.
Use `Ctrl+Shift+Alt+L` for debug `--verbose` (extension, prefix scanner, filter counts).

## steecleditor snippets
`steecleditor` now includes language snippets available via completion (or `Ctrl+Shift+I` for steelconf snippets):

- Common trigger convention (when language syntax allows it):
  - `main`: entrypoint skeleton
  - `func`: function/method skeleton
  - `test`: test skeleton
- Exceptions:
  - `JavaScript/TypeScript` keep `jest` and `vitest` for framework-specific tests.
  - `Ruby` keeps `rspec` for backward compatibility (in addition to `test`).
  - `Algol` keeps `begin` as canonical block starter.
  - `CoffeeScript` and `PHP` currently expose `func` only (no canonical entrypoint in typical scripts).
  - `Haskell` currently exposes `main` only.

### Snippet aliases

| Alias | Canonical |
|---|---|
| `rspec` (Ruby) | `test` |
| `spec` (Ruby) | `test` |
| `example` (Ruby) | `test` |

<!-- SNIPPET-SNAPSHOT:START -->
```text
Ruby: class, example, func, main, rspec, spec, test
Kotlin: func, main, test
Swift: func, main, test
Dart: func, main, test
Solidity: func, main, test
PowerShell: func, main, test
Makefile: func, main, test
WGSL: func, main, test
OpenCL C: func, main, test
Hack: func, main, test
Apex: func, main, test
Go: func, main, test
Zig: main
Java: func, main, test
HolyC: assert, const, func, main, static, struct, test
Pascal: func, main, test
Algol: begin
Haskell: main
Lua: func, main, test
JavaScript: func, jest, vitest
TypeScript: func, jest, vitest
CoffeeScript: func
PHP: func
Rust: func, main, test
```
<!-- SNIPPET-SNAPSHOT:END -->

- Ruby / RSpec:
  - `test`, `example`, `spec` or `rspec` -> `describe ... it ... expect(...)`
  - `func` -> Ruby method skeleton
  - `main` -> `if __FILE__ == $0`
  - `class` -> Ruby class skeleton
- Go:
  - `main` -> `package main` + `func main()`
  - `func` -> function skeleton
  - `test` -> `func TestXxx(t *testing.T)`
- Kotlin:
  - `main`, `func`, `test` snippets
- Swift:
  - `main`, `func`, `test` snippets
- Dart:
  - `main`, `func`, `test` snippets
- Solidity:
  - `main`, `func`, `test` snippets
- PowerShell:
  - `main`, `func`, `test` snippets (`It ... Should ...`)
- Makefile:
  - `main`, `func` (variable), `test` (target) snippets
- WGSL:
  - `main` -> compute entrypoint skeleton
  - `func` -> function skeleton
  - `test` -> boolean test helper
- OpenCL C:
  - `main` -> kernel skeleton
  - `func` -> inline helper
  - `test` -> test helper
- Hack:
  - `main` -> `<<__EntryPoint>>` skeleton
  - `func` -> typed function skeleton
  - `test` -> `invariant(...)` style helper
- Apex:
  - `main` -> class skeleton
  - `func` -> static method skeleton
  - `test` -> `@IsTest` class/method skeleton
- Zig:
  - `main` -> `pub fn main() !void`
- Java:
  - `main` -> class + `public static void main`
  - `func` -> method skeleton
  - `test` -> JUnit test method skeleton
- HolyC:
  - `main` -> `U0 Main()`
  - `func` -> function skeleton
  - `test` -> test function skeleton
  - `assert` -> assertion helper block
  - `const` -> constant string
  - `static` -> static array
  - `struct` -> class/struct skeleton
- Pascal:
  - `main` -> program skeleton
  - `func` -> function skeleton
  - `test` -> test procedure skeleton
- Algol:
  - `begin` -> basic begin/end block
- Haskell:
  - `main` -> `main :: IO ()`
- Lua:
  - `main` -> local function main
  - `func` -> local function skeleton
  - `test` -> lightweight test function
- JavaScript / TypeScript:
  - `func` -> function skeleton
  - `test` -> use `jest` / `vitest` shortcuts
- CoffeeScript:
  - `func` -> `name = (args) ->`
- PHP:
  - `func` -> PHP function skeleton

## Support matrix

| Language | Highlighting | Completion | Snippets | Symbols |
|---|---|---|---|---|
| steelconf | yes | blocks/directives/snippets | steelconf-only picker | bake/workspace structure |
| C / C++ | yes (comments, doc comments, preproc, raw strings) | keywords (+ shared C/HolyC memory builtins for C) | no | functions/struct/class/namespace/enum |
| Python | yes (triple-quote, f-string) | keywords+builtins | no | def/class |
| Perl | yes (regex, heredoc, vars) | keywords+builtins | no | sub |
| sh/zsh | yes (line comments, heredoc) | keywords+builtins (POSIX + Bash + Zsh union) | no | function |
| WGSL | yes (C-like comments/strings) | keywords+builtins | yes | fn/struct |
| OpenCL C | yes (C-like comments/strings) | keywords+builtins | yes | kernel/function |
| Hack | yes (C-like comments/strings) | keywords+builtins | yes | function/class |
| Apex | yes (C-like comments/strings) | keywords+builtins | yes | trigger/class/method |
| Go | yes (raw/backtick strings) | keywords+builtins | yes | func/type |
| Rust | yes (raw strings, byte/raw forms) | keywords+builtins | yes (`#[test]`) | fn/struct/enum/trait/impl |
| Ruby | yes (heredoc, interpolation, `%` literals, symbols, vars) | keywords+builtins | yes (class, RSpec) | def/class/module |
| Pascal | yes (line + nested block comments) | keywords+builtins | yes | program/procedure/function |
| Algol | yes (line + block comments) | keywords+builtins | yes | program/procedure/function |
| HolyC | yes (C-like) | keywords+builtins | yes | C-like function symbols |
| Haskell | yes (nested block comments) | keywords+builtins | yes | signature/definition |
| Lua | yes (long strings/comments) | keywords+builtins | yes | function |
| JavaScript | yes (template strings + regex) | keywords+builtins | yes (func, jest, vitest) | function/class/lambda |
| TypeScript | yes (template strings + regex) | keywords+builtins | yes (func, jest, vitest) | function/class/lambda |
| CoffeeScript | yes (### block comments, triple strings, regex) | keywords+builtins | yes | function/class/lambda |
| PHP | yes (heredoc/nowdoc, vars, tags/directives) | keywords+builtins | yes | function/class |

### Phase2 coverage

<!-- PHASE2-COV:START -->
| Language | Advanced highlighting | Symbols | Snippets | QA tests | Known gaps |
|---|---|---|---|---|---|
| Kotlin | yes (base C-like + phase2 non-regression) | class/fun | main/func/test | `collect_symbols_for_new_languages`, `symbols_false_positives_phase2`, `phase2_highlighting_non_regression` | no dedicated multiline literals edge-case test yet |
| Swift | yes (base C-like + phase2 non-regression) | struct/class/func | main/func/test | `collect_symbols_for_new_languages`, `symbols_false_positives_phase2`, `phase2_highlighting_non_regression` | protocol/extension symbol nuances remain heuristic |
| Dart | yes (base C-like + phase2 non-regression) | class/func + extension/mixin | main/func/test | `collect_symbols_phase2_extended_power_shell_dart`, `collect_symbols_phase2_false_positives`, `phase2_highlighting_non_regression` | generic extension edge cases still regex-based |
| Solidity | yes (`mapping(...)`, phase2 non-regression) | contract/event/function | main/func/test | `collect_symbols_for_new_languages`, `symbols_false_positives_phase2`, `phase2_highlighting_non_regression` | modifier/library coverage still partial in symbols |
| PowerShell | yes (here-strings + phase2 non-regression) | function + class/enum | main/func/test | `collect_symbols_phase2_extended_power_shell_dart`, `collect_symbols_phase2_false_positives`, `phase2_highlighting_non_regression`, `render_line_here_string_roundtrip`, `render_line_here_string_double_quote_roundtrip` | parser remains line-oriented for complex scriptblocks |
| Makefile | yes (`$(...)` vars, tab recipes) | target | main/func/test | `collect_symbols_for_new_languages`, `collect_symbols_makefile_false_positives`, `collect_symbols_makefile_pattern_targets`, `symbols_false_positives_phase2`, `phase2_highlighting_non_regression` | pattern/static pattern targets are simplified |
<!-- PHASE2-COV:END -->

### Phase2 non-goals

<!-- PHASE2-NONGOALS:START -->
- No AST parser per language in phase2; symbol detection remains regex/pattern based.
- No full shell/PowerShell parser state machine for every nested quoting edge case.
- No exhaustive Makefile semantics (pattern-specific variants, include graph resolution, eval/call expansion).
- No strict framework-specific snippet style unification when idiomatic labels are clearer (example: XCTestCase, group/test, Make target).
<!-- PHASE2-NONGOALS:END -->

### How to validate phase2 quickly

- `cargo test --bin steecleditor phase2_highlighting_non_regression`
- `cargo test --bin steecleditor collect_symbols_phase2_extended_power_shell_dart`
- `cargo test --bin steecleditor collect_symbols_makefile_static_pattern_targets`
- `cargo test --bin steecleditor phase2_coverage_snapshot_matches_docs`

Additional base support (detection + keywords/builtins completion):
Kotlin, Swift, Dart, Elixir, Erlang, Clojure, F#, R, Julia, MATLAB/Octave, Scala, Groovy, Nim, Crystal, Fortran, COBOL, Ada, Assembly (x86/ARM), V, Solidity, Move, VHDL, Verilog/SystemVerilog, Prolog, Scheme, Smalltalk, Tcl, PowerShell, Fish, Makefile.

### Base vs advanced (new 30)

<!-- NEW30-CAP:START -->
```text
Kotlin: base=yes advanced=yes
Swift: base=yes advanced=yes
Dart: base=yes advanced=yes
Elixir: base=yes advanced=no
Erlang: base=yes advanced=no
Clojure: base=yes advanced=no
F#: base=yes advanced=no
R: base=yes advanced=no
Julia: base=yes advanced=no
MATLAB/Octave: base=yes advanced=no
Scala: base=yes advanced=no
Groovy: base=yes advanced=no
Nim: base=yes advanced=no
Crystal: base=yes advanced=no
Fortran: base=yes advanced=no
COBOL: base=yes advanced=no
Ada: base=yes advanced=no
Assembly: base=yes advanced=no
V: base=yes advanced=no
Solidity: base=yes advanced=yes
Move: base=yes advanced=no
VHDL: base=yes advanced=no
Verilog/SystemVerilog: base=yes advanced=no
Prolog: base=yes advanced=no
Scheme: base=yes advanced=no
Smalltalk: base=yes advanced=no
Tcl: base=yes advanced=no
PowerShell: base=yes advanced=yes
Fish: base=yes advanced=no
Makefile: base=yes advanced=yes
```
<!-- NEW30-CAP:END -->

## Known gaps

- `sh/zsh`: lexer is line-oriented; some complex heredoc/body edge cases are not fully tracked across all nested constructs.
- `Pascal`: nested comments are supported for `(* ... *)`, but mixed comment styles in legacy dialects can still mis-highlight.
- `Algol` and `HolyC`: highlighting/symbol extraction remains heuristic and may miss uncommon syntax forms.
- `CoffeeScript`: advanced interpolation/regex ambiguities can still produce imperfect token boundaries in rare cases.
- Symbol navigation is pattern-based (not AST-based), so it can include false positives in highly dynamic or generated code.

## How to run QA locally

- Run focused editor tests:
  - `cargo test --bin steecleditor`
- Run only snapshot/ordering/symbol-regression tests quickly:
  - `cargo test --bin steecleditor debug_`
  - `cargo test --bin steecleditor completion_order_is_stable`
  - `cargo test --bin steecleditor collect_symbols_edge_cases`
- Run highlighting benchmark on large fixtures (`tests/fixtures/large.js`, `tests/fixtures/large.rs`, `tests/fixtures/large.php`):
  - `cargo test --bin steecleditor benchmark_highlighting_large_files -- --ignored --nocapture`
- CI:
  - `.github/workflows/steecleditor.yml` runs `cargo test --bin steecleditor` on push/PR.
  - Snapshot sync check runs fast via `cargo test --bin steecleditor snippets_doc_snapshot_matches_language_data`.
  - Manual benchmark run is available via `workflow_dispatch` with `run_benchmark=true`.
- Regenerate snippet snapshot block (manual utility):
  - `cargo test --bin steecleditor regenerate_snippet_snapshot_block -- --ignored`
- Debug completion source in editor:
  - `Ctrl+Shift+L` for standard debug panel
  - `Ctrl+Shift+Alt+L` for verbose debug (extension + prefix scanner + filters + shell dialect)
  - Temporary shell dialect override for debug/completion: `STEECLEDITOR_SHELL_DIALECT=posix|bash|zsh|union`

## Vim / Neovim
Add to `~/.vimrc` or `~/.config/nvim/init.vim`:
```
augroup steelconf_ft
  autocmd!
  autocmd BufRead,BufNewFile steelconf,*.muf setfiletype steelconf
augroup END
```
Then add to `~/.vim/ftplugin/steelconf.vim` or `~/.config/nvim/ftplugin/steelconf.vim`:
```
setlocal tabstop=2
setlocal shiftwidth=2
setlocal expandtab
setlocal softtabstop=2
```

## Helix
Add to `~/.config/helix/languages.toml`:
```
[[language]]
name = "steelconf"
file-types = ["steelconf", "muf"]
indent = { tab-width = 2, unit = "  " }
```

## micro
Create `~/.config/micro/ftdetect/steelconf.yaml`:
```
filetype: steelconf
detect:
  filename: steelconf
  extension: muf
```
Then create `~/.config/micro/ftplugin/steelconf.lua`:
```
local config = import("micro/config")
config.MakeLocalOption("tabsize", 2)
config.MakeLocalOption("tabstospaces", true)
```

## nano
Add to `~/.nanorc` (global settings):
```
set tabsize 2
set tabstospaces
```

## Emacs
Add to your init file (`~/.emacs` or `~/.emacs.d/init.el`):
```
(add-to-list 'auto-mode-alist '("steelconf\\'" . fundamental-mode))
(add-to-list 'auto-mode-alist '("\\.muf\\'" . fundamental-mode))
(add-hook 'fundamental-mode-hook
          (lambda ()
            (when (or (string-equal (file-name-nondirectory (or (buffer-file-name) "")) "steelconf")
                      (and (buffer-file-name) (string-match "\\.muf\\'" (buffer-file-name))))
              (setq-local indent-tabs-mode nil)
              (setq-local tab-width 2)
              (setq-local standard-indent 2))))
```

## Zed
Add to `~/.config/zed/settings.json`:
```
{
  "languages": {
    "Plain Text": {
      "file_types": ["steelconf", "muf"],
      "tab_size": 2,
      "indent_width": 2,
      "soft_wrap": "none"
    }
  }
}
```

## Sublime Text
Create `~/.config/sublime-text/Packages/User/steelconf.sublime-settings`:
```
{
  "tab_size": 2,
  "translate_tabs_to_spaces": true
}
```
Create `~/.config/sublime-text/Packages/User/steelconf.sublime-syntax`:
```
%YAML 1.2
---
name: steelconf
file_extensions:
  - steelconf
  - muf
scope: text.steelconf
contexts:
  main: []
```

## JetBrains IDEs
1) Settings -> Editor -> File Types -> Text -> add `steelconf` and `*.muf`.
2) Settings -> Editor -> Code Style -> Plain Text -> set Tab size = 2, Indent = 2, Tabs = Spaces.
