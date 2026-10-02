use std::fs;
use std::path::PathBuf;

#[no_mangle]
pub extern "C" fn cleo_diagnostic_init() {
    let mut path = match std::env::var_os("HOME") {
        Some(home) => PathBuf::from(home),
        None => return,
    };

    path.push("Documents");
    path.push("CLEO");

    if fs::create_dir_all(&path).is_err() {
        return;
    }

    path.push("LIVE_CONTAINER_DIAGNOSTIC.txt");

    let _ = fs::write(
        path,
        b"CLEO LiveContainer diagnostic loaded successfully.\n",
    );
}

#[used]
#[cfg_attr(target_os = "ios", link_section = "__DATA,__mod_init_func")]
static INIT: extern "C" fn() = {
    extern "C" fn init() {
        cleo_diagnostic_init();
    }

    init
};