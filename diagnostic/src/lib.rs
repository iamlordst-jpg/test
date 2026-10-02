mod csi;

use std::fs;
use std::path::{Path, PathBuf};

fn write_log(path: &Path, text: &str) {
    let _ = fs::write(path, text);
}

fn append_log(path: &Path, text: &str) {
    use std::io::Write;
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{text}");
    }
}

fn scan_scripts(scripts_dir: &Path, log_path: &Path) {
    let mut entries: Vec<PathBuf> = match fs::read_dir(scripts_dir) {
        Ok(read_dir) => read_dir
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| {
                matches!(
                    path.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()),
                    Some(ext) if ext == "csi" || ext == "csa"
                )
            })
            .collect(),
        Err(_) => return,
    };

    entries.sort();

    append_log(log_path, "CLEO CSI/CSA Loader");
    append_log(log_path, "===================");
    append_log(log_path, &format!("Found {} CSI/CSA script(s)", entries.len()));

    for path in entries {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        let kind = if ext == "csa" { "CSA" } else { "CSI" };
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("<unnamed>");

        append_log(log_path, &format!("\n========== {name} =========="));
        append_log(log_path, &format!("Type: {kind}"));

        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(err) => {
                append_log(log_path, &format!("[ERROR] read failed: {err}"));
                continue;
            }
        };

        append_log(log_path, &format!("Size: {} bytes", bytes.len()));

        let mut lines = Vec::new();
        match csi::inspect(&bytes, kind, &mut |line| lines.push(line)) {
            Ok(info) => {
                append_log(log_path, &format!("Instructions: {}", info.instructions));
                append_log(log_path, "Status: decoded successfully");
                for line in lines {
                    append_log(log_path, &line);
                }
            }
            Err(err) => {
                append_log(log_path, &format!("Status: decode failed"));
                append_log(log_path, &format!("[ERROR] {err}"));
            }
        }

        append_log(log_path, "========== SCRIPT END ==========");
    }
}

#[no_mangle]
pub extern "C" fn cleo_diagnostic_init() {
    let home = match std::env::var_os("HOME") {
        Some(home) => PathBuf::from(home),
        None => return,
    };

    let cleo = home.join("Documents").join("CLEO");
    let scripts = cleo.join("scripts");
    let logs = cleo.join("logs");
    let config = cleo.join("config");
    let plugins = cleo.join("plugins");

    if fs::create_dir_all(&scripts).is_err()
        || fs::create_dir_all(&logs).is_err()
        || fs::create_dir_all(&config).is_err()
        || fs::create_dir_all(&plugins).is_err()
    {
        return;
    }

    write_log(
        &cleo.join("LIVE_CONTAINER_DIAGNOSTIC.txt"),
        "CLEO LiveContainer diagnostic loaded successfully.\n",
    );

    write_log(
        &config.join("runtime.txt"),
        "CLEO Custom Runtime\nVersion: 0.6.0\nCSI/CSA loader: enabled\nJailbreak: not required\n",
    );

    let startup = logs.join("startup.log");
    write_log(
        &startup,
        "CLEO Custom Runtime\n====================\nVersion: 0.6.0\nScript engine: CSI/CSA\nSandbox mode: enabled\nJailbreak required: no\n",
    );

    let script_log = logs.join("scripts.log");
    write_log(&script_log, "");
    scan_scripts(&scripts, &script_log);
}

#[used]
#[cfg_attr(target_os = "ios", link_section = "__DATA,__mod_init_func")]
static INIT: extern "C" fn() = {
    extern "C" fn init() {
        cleo_diagnostic_init();
    }
    init
};
