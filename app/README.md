# MkvFontMux GUI

Flutter desktop app (Windows, macOS, Linux) on top of `mfm-core`, connected through [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge) 2.13.

## Build

Requires Flutter 3.47+, stable Rust and a C++17 compiler. On Linux, also `ninja-build libgtk-3-dev clang cmake pkg-config`.

```sh
cd app
flutter run -d windows        # or macos / linux
flutter build windows --release
```

Paths passed on the command line are added as jobs at startup.

## Layout

| Path | Description |
| --- | --- |
| `rust/` | `mfm_bridge`: the Rust API exposed to Dart (`src/api/`) |
| `lib/src/rust/` | Generated Dart bindings (do not edit) |
| `lib/state/` | Riverpod state: settings, font library, jobs |
| `lib/ui/` | Pages: jobs, font library, settings |
| `lib/l10n/` | English and Chinese strings |

After changing `rust/src/api/`, regenerate the bindings:

```sh
cargo install flutter_rust_bridge_codegen@2.13.0
flutter_rust_bridge_codegen generate
```

## Tests

```sh
flutter test integration_test -d windows \
  --dart-define=MFM_E2E_DIR=<folder with .mkv and .ass files> \
  --dart-define=MFM_SHOT_DIR=<folder for page screenshots>
```

Both defines are optional; without `MFM_E2E_DIR` only the smoke tests run. The end-to-end test uses the built-in muxer and writes to an `out_gui` subfolder.
