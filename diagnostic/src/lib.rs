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

fn parse_print_command(line: &str) -> Option<String> {
    let line = line.trim();

    if !line.starts_with("PRINT ") {
        return None;
    }

    let value = line[6..].trim();

    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return Some(value[1..value.len() - 1].to_string());
    }

    None
}

fn execute_script(root: &Path, script_path: &Path, runtime_log: &mut String) {
    let contents = match fs::read_to_string(script_path) {
        Ok(contents) => contents,
        Err(_) => {
            runtime_log.push_str("[error] Could not read script\n");
            return;
        }
    };

    let script_name = script_path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();

    runtime_log.push_str(&format!("[script] {}\n", script_name));

    for line in contents.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if let Some(message) = parse_print_command(line) {
            runtime_log.push_str("[command] PRINT\n");
            runtime_log.push_str(&format!("[text] {}\n", message));
        } else {
            runtime_log.push_str(&format!("[unknown] {}\n", line));
        }
    }

    runtime_log.push('\n');
}

fn load_scripts(root: &Path) {
    let scripts_dir = root.join("scripts");
    let runtime_log_path = root.join("logs").join("runtime.log");

    let mut runtime_log = String::from(
        "CLEO Custom Runtime\n\
====================\n\n",
    );

    let entries = match fs::read_dir(&scripts_dir) {
        Ok(entries) => entries,
        Err(_) => {
            runtime_log.push_str("[error] Could not read scripts directory.\n");
            write_file(&runtime_log_path, runtime_log.as_bytes());
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
            scripts.push(path);
        }
    }

    scripts.sort();

    runtime_log.push_str(&format!(
        "Found {} script(s)\n\n",
        scripts.len()
    ));

    for script in scripts {
        execute_script(root, &script, &mut runtime_log);
    }

    write_file(&runtime_log_path, runtime_log.as_bytes());
}

pub fn cleo_init() {
    let Some(root) = cleo_root() else {
        return;
    };

    create_directory(&root);
    create_directory(&root.join("scripts"));
    create_directory(&root.join("config"));
    create_directory(&root.join("plugins"));
    create_directory(&root.join("logs"));

    write_file(
        &root.join("logs").join("startup.log"),
        b"CLEO custom runtime initialized successfully.\n",
    );

    write_file(
        &root.join("config").join("runtime.txt"),
        b"CLEO Custom Runtime\n\
Version: 0.3.0\n\
Platform: iOS arm64\n\
Loader: LiveContainer\n\
Script discovery: enabled\n\
Script execution: enabled\n",
    );

    load_scripts(&root);

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