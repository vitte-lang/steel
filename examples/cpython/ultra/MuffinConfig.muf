!muf 4
;; MuffinConfig.muf — CPython Ultra Project (Library Catalog Simulator)
;; Usage:
;;   muffin run --root . --file MuffinConfig.muf --bake app
;;   muffin run --root . --file MuffinConfig.muf --bake tests

[workspace]
  .set name "cpython-ultra"
  .set root "."
  .set target_dir "target"
  .set profile "debug"
..

[profile debug]
  .set opt 0
  .set debug 1
..

[profile release]
  .set opt 2
  .set debug 0
..

[tool python]
  .exec "python3"
..

[tool sh]
  .exec "sh"
..

[bake app]
  .make py_src cglob "cli/*.py"
  [run python]
    .set "-u" 1
    .set "-m" "cli.main"
  ..
  .output exe "target/out/app.run"
..

[bake tests]
  .make py_src cglob "tests/*.py"
  [run python]
    .set "-u" 1
    .set "-m" "tests.run"
  ..
  .output exe "target/out/tests.run"
..

[bake install_nuitka]
  .make py_src cglob "cli/*.py"
  [run sh]
    .set "-c" "python3 -m venv .venv && .venv/bin/python -m pip install --upgrade pip nuitka && touch target/out/install_nuitka.done"
  ..
  .output exe "target/out/install_nuitka.done"
..

[bake install_pyinstaller]
  .make py_src cglob "cli/*.py"
  [run sh]
    .set "-c" "python3 -m venv .venv && .venv/bin/python -m pip install --upgrade pip pyinstaller && touch target/out/install_pyinstaller.done"
  ..
  .output exe "target/out/install_pyinstaller.done"
..

[bake mac_app_nuitka]
  .make py_src cglob "cli/*.py"
  [run sh]
    .set "-c" ".venv/bin/python -m nuitka --standalone --onefile --output-dir=target/out --output-filename=cpython_ultra_macos cli/main.py"
  ..
  .output exe "target/out/cpython_ultra_macos"
..

[bake mac_app_pyinstaller]
  .make py_src cglob "cli/*.py"
  [run sh]
    .set "-c" ".venv/bin/python -m PyInstaller --onefile --name cpython_ultra_macos --distpath target/out --workpath target/pyinstaller cli/main.py"
  ..
  .output exe "target/out/cpython_ultra_macos"
..

[export]
  .ref app
  .ref tests
  .ref install_nuitka
  .ref install_pyinstaller
  .ref mac_app_nuitka
  .ref mac_app_pyinstaller
..
