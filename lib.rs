//! Minimal LiveContainer diagnostic dylib.
//!
//! This intentionally does NOT initialize CLEO, install hooks, inspect GTA,
//! or reference any jailbreak paths. Its only job is to prove that a Rust
//! iOS dylib can be loaded by LiveContainer and safely perform sandbox I/O.

use ctor::ctor;
use std::fs;
use std::path::PathBuf;

#[ctor]
fn livecontainer_diagnostic_init() {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    let cleo_dir = home.join("Documents").join("CLEO");
    let _ = fs::create_dir_all(&cleo_dir);

    let marker = cleo_dir.join("LIVE_CONTAINER_DIAGNOSTIC.txt");
    let contents = concat!(
        "CLEO LiveContainer diagnostic loaded successfully.\n",
        "This build intentionally performs no GTA hooks.\n"
    );
    let _ = fs::write(marker, contents);
}
