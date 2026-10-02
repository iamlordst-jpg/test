use std::fs;
use std::path::{Path, PathBuf};

fn cleo_root() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join("Documents").join("CLEO"))
}

fn create_directory(path: &Path) {
    let _ = fs::create_dir_all(path);
}

fn write_file(path: &Path, contents: &[u8]) {
    let _ = fs::write(path, contents);
}

pub fn cleo_init() {
    let Some(root) = cleo_root() else {
        return;
    };

    // Runtime directories.
    create_directory(&root);
    create_directory(&root.join("scripts"));
    create_directory(&root.join("config"));
    create_directory(&root.join("plugins"));
    create_directory(&root.join("logs"));

    // Startup marker.
    write_file(
        &root.join("logs").join("startup.log"),
        b"CLEO custom runtime initialized successfully.\n",
    );

    // Runtime version information.
    write_file(
        &root.join("config").join("runtime.txt"),
        b"CLEO Custom Runtime\n\
Version: 0.1.0\n\
Platform: iOS arm64\n\
Loader: LiveContainer\n",
    );

    // Temporary diagnostic marker.
    write_file(
        &root.join("LIVE_CONTAINER_DIAGNOSTIC.txt"),
        b"CLEO LiveContainer diagnostic loaded successfully.\n",
    );
}

#[no_mangle]
pub extern "C" fn cleo_diagnostic_init() {
    cleo_init();
}

#[used]
#[cfg_attr(target_os = "ios", link_section = "__DATA,__mod_init_func")]
static INIT: extern "C" fn() = {
    extern "C" fn init() {
        cleo_init();
    }

    init
};