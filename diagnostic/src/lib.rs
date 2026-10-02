use std::fs;
use std::path::PathBuf;

fn cleo_runtime_init() {
    let home = match std::env::var_os("HOME") {
        Some(home) => PathBuf::from(home),
        None => return,
    };

    let cleo_dir = home.join("Documents").join("CLEO");

    // Create the basic CLEO runtime directory structure.
    if fs::create_dir_all(cleo_dir.join("scripts")).is_err() {
        return;
    }

    if fs::create_dir_all(cleo_dir.join("config")).is_err() {
        return;
    }

    if fs::create_dir_all(cleo_dir.join("runtime")).is_err() {
        return;
    }

    // Write a runtime status file.
    let status = cleo_dir.join("runtime").join("status.txt");

    let _ = fs::write(
        status,
        b"CLEO runtime initialized successfully.\n\
         Platform: iOS\n\
         Architecture: arm64\n\
         Loader: LiveContainer/TweakLoader\n",
    );

    // Keep the original diagnostic marker too.
    let diagnostic = cleo_dir.join("LIVE_CONTAINER_DIAGNOSTIC.txt");

    let _ = fs::write(
        diagnostic,
        b"CLEO LiveContainer diagnostic loaded successfully.\n",
    );
}

#[used]
#[cfg_attr(target_os = "ios", link_section = "__DATA,__mod_init_func")]
static INIT: extern "C" fn() = {
    extern "C" fn init() {
        cleo_runtime_init();
    }

    init
};