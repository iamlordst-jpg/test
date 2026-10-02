# CLEO LiveContainer diagnostic build

This is the next isolation test for the supplied CLEO iOS 2.6.0 source.

It deliberately does **not** compile or load the existing CLEO engine. It is a tiny
ARM64 iOS dylib that has a load constructor and performs one sandbox-safe file operation.

## Why this is separate

The original 2.6.0 constructor calls `logging::init()`, `hook::can_hook()`, `meta::init()`,
and `game::init()` immediately when the dylib is loaded. Your original dylib crashes
LiveContainer/GTA, while the empty diagnostic dylib does not. This test isolates the
next layer without touching GTA's memory or fixed addresses.

## GitHub Actions

Copy this folder's `.github/workflows/build-livecontainer-diagnostic.yml` into your repo,
then run the workflow from GitHub Actions. The resulting artifact contains `CLEO.dylib`
and a plist for `gta3sa`, `gtasager`, and `gtasa`.

The ARM64 iOS target is the native `aarch64-apple-ios` Rust target; Rust documents it as
the device target and notes that it requires an Xcode iPhoneOS SDK on macOS.
