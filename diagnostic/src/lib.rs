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

fn discover_scripts(root: &Path) {
    let scripts_dir = root.join("scripts");

    let mut log = String::from(
        "CLEO Script Loader\n\
==================\n",
    );

    let entries = match fs::read_dir(&scripts_dir) {
        Ok(entries) => entries,
        Err(_) => {
            log.push_str("\nUnable to read scripts directory.\n");
            write_file(&root.join("logs").join("scripts.log"), log.as_bytes());
            return;
        }
    };

    let mut scripts = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(extension) = path.extension() else {
            continue;
        };

        if extension.to_string_lossy().eq_ignore_ascii_case("cs") {
            if let Some(name) = path.file_name() {
                scripts.push(name.to_string_lossy().to_string());
            }
        }
    }

    scripts.sort();

    log.push_str(&format!("\nFound {} script(s)\n\n", scripts.len()));

    for script in &scripts {
        log.push_str(script);
        log.push('\n');
    }

    write_file(&root.join("logs").join("scripts.log"), log.as_bytes());
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

    // Runtime information.
    write_file(
        &root.join("config").join("runtime.txt"),
        b"CLEO Custom Runtime\n\
Version: 0.2.0\n\
Platform: iOS arm64\n\
Loader: LiveContainer\n\
Script discovery: enabled\n",
    );

    // Discover .cs scripts.
    discover_scripts(&root);

    // Keep the original diagnostic marker.
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