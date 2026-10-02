use std::fs;
use std::path::PathBuf;

fn cleo_runtime_init() {
    let home = match std::env::var_os("HOME") {
        Some(home) => PathBuf::from(home),
        None => return,
    };

    let cleo_dir = home.join("Documents").join("CLEO");

    // Automatically create the CLEO runtime directory.
    if fs::create_dir_all(&cleo_dir).is_err() {
        return;
    }

    // Create the directories we'll use for the runtime later.
    let _ = fs::create_dir_all(cleo_dir.join("scripts"));
    let _ = fs::create_dir_all(cleo_dir.join("config"));
    let _ = fs::create_dir_all(cleo_dir.join("runtime"));

    // Diagnostic marker proving the dylib initializer executed.
    let _ = fs::write(
        cleo_dir.join("LIVE_CONTAINER_DIAGNOSTIC.txt"),
        b"CLEO LiveContainer diagnostic loaded successfully.\n",
    );

    // Runtime status.
    let _ = fs::write(
        cleo_dir.join("runtime").join("status.txt"),
        b"CLEO runtime initialized successfully.\n\
Platform: iOS\n\
Architecture: arm64\n\
Loader: LiveContainer/TweakLoader\n",
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