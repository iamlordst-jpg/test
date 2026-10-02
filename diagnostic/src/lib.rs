use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum Comparison {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
}

#[derive(Debug)]
enum Instruction {
    Print(String),
    Wait(u64),

    Set(String, i64),
    Add(String, i64),
    Sub(String, i64),
    Inc(String),
    Dec(String),
    PrintVar(String),

    Goto(String),

    If {
        variable: String,
        comparison: Comparison,
        value: i64,
        label: String,
    },
}

struct ScriptRuntime {
    variables: HashMap<String, i64>,
}

impl ScriptRuntime {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    fn get_variable(&self, name: &str) -> i64 {
        self.variables
            .get(name)
            .copied()
            .unwrap_or(0)
    }

    fn compare(
        &self,
        variable: &str,
        comparison: &Comparison,
        value: i64,
    ) -> bool {
        let current = self.get_variable(variable);

        match comparison {
            Comparison::Equal => current == value,
            Comparison::NotEqual => current != value,
            Comparison::Greater => current > value,
            Comparison::Less => current < value,
            Comparison::GreaterOrEqual => current >= value,
            Comparison::LessOrEqual => current <= value,
        }
    }

    fn execute(
        &mut self,
        instruction: &Instruction,
        log: &mut String,
    ) {
        match instruction {
            Instruction::Print(message) => {
                log.push_str("[PRINT] ");
                log.push_str(message);
                log.push('\n');
            }

            Instruction::Wait(milliseconds) => {
                log.push_str(&format!(
                    "[WAIT] {} ms\n",
                    milliseconds
                ));

                thread::sleep(
                    Duration::from_millis(*milliseconds)
                );
            }

            Instruction::Set(name, value) => {
                self.variables
                    .insert(name.clone(), *value);

                log.push_str(&format!(
                    "[SET] {} = {}\n",
                    name, value
                ));
            }

            Instruction::Add(name, value) => {
                let variable =
                    self.variables
                        .entry(name.clone())
                        .or_insert(0);

                *variable += *value;

                log.push_str(&format!(
                    "[ADD] {} = {}\n",
                    name, *variable
                ));
            }

            Instruction::Sub(name, value) => {
                let variable =
                    self.variables
                        .entry(name.clone())
                        .or_insert(0);

                *variable -= *value;

                log.push_str(&format!(
                    "[SUB] {} = {}\n",
                    name, *variable
                ));
            }

            Instruction::Inc(name) => {
                let variable =
                    self.variables
                        .entry(name.clone())
                        .or_insert(0);

                *variable += 1;

                log.push_str(&format!(
                    "[INC] {} = {}\n",
                    name, *variable
                ));
            }

            Instruction::Dec(name) => {
                let variable =
                    self.variables
                        .entry(name.clone())
                        .or_insert(0);

                *variable -= 1;

                log.push_str(&format!(
                    "[DEC] {} = {}\n",
                    name, *variable
                ));
            }

            Instruction::PrintVar(name) => {
                let value =
                    self.get_variable(name);

                log.push_str(&format!(
                    "[PRINTVAR] {} = {}\n",
                    name, value
                ));
            }

            Instruction::Goto(label) => {
                log.push_str(&format!(
                    "[GOTO] {}\n",
                    label
                ));
            }

            Instruction::If {
                variable,
                comparison,
                value,
                label,
            } => {
                let result =
                    self.compare(
                        variable,
                        comparison,
                        *value,
                    );

                log.push_str(&format!(
                    "[IF] {} -> {}\n",
                    variable,
                    result
                ));

                if result {
                    log.push_str(&format!(
                        "[IF] jumping to {}\n",
                        label
                    ));
                }
            }
        }
    }
}

fn cleo_root() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;

    Some(
        PathBuf::from(home)
            .join("Documents")
            .join("CLEO"),
    )
}

fn create_directory(path: &Path) {
    let _ = fs::create_dir_all(path);
}

fn write_file(path: &Path, contents: &[u8]) {
    let _ = fs::write(path, contents);
}

fn parse_comparison(
    operator: &str,
) -> Option<Comparison> {
    match operator {
        "==" => Some(Comparison::Equal),
        "!=" => Some(Comparison::NotEqual),
        ">" => Some(Comparison::Greater),
        "<" => Some(Comparison::Less),
        ">=" => Some(Comparison::GreaterOrEqual),
        "<=" => Some(Comparison::LessOrEqual),
        _ => None,
    }
}

fn parse_instruction(
    line: &str,
) -> Result<Option<Instruction>, String> {
    let line = line.trim();

    if line.is_empty() {
        return Ok(None);
    }

    if line.starts_with("//") {
        return Ok(None);
    }

    if line.eq_ignore_ascii_case("END") {
        return Ok(None);
    }

    // ---------------------------------------------------------
    // LABEL
    // ---------------------------------------------------------

    if line.starts_with(':') {
        return Ok(None);
    }

    // ---------------------------------------------------------
    // PRINTVAR variable
    // ---------------------------------------------------------

    if line.len() >= 8
        && line[..8].eq_ignore_ascii_case("PRINTVAR")
    {
        let name = line[8..].trim();

        if name.is_empty() {
            return Err(
                "PRINTVAR requires a variable name"
                    .to_string()
            );
        }

        return Ok(Some(
            Instruction::PrintVar(
                name.to_string()
            )
        ));
    }

    // ---------------------------------------------------------
    // PRINT "message"
    // ---------------------------------------------------------

    if line.len() >= 5
        && line[..5].eq_ignore_ascii_case("PRINT")
    {
        let rest = line[5..].trim();

        if rest.len() >= 2
            && rest.starts_with('"')
            && rest.ends_with('"')
        {
            let message =
                rest[1..rest.len() - 1]
                    .to_string();

            return Ok(Some(
                Instruction::Print(message)
            ));
        }

        return Err(
            "PRINT requires a quoted string"
                .to_string()
        );
    }

    // ---------------------------------------------------------
    // WAIT milliseconds
    // ---------------------------------------------------------

    if line.len() >= 4
        && line[..4].eq_ignore_ascii_case("WAIT")
    {
        let rest = line[4..].trim();

        let milliseconds =
            rest.parse::<u64>()
                .map_err(|_| {
                    "WAIT requires an integer number of milliseconds"
                        .to_string()
                })?;

        return Ok(Some(
            Instruction::Wait(milliseconds)
        ));
    }

    // ---------------------------------------------------------
    // SET variable value
    // ---------------------------------------------------------

    if line.len() >= 3
        && line[..3].eq_ignore_ascii_case("SET")
    {
        let parts: Vec<&str> =
            line[3..]
                .split_whitespace()
                .collect();

        if parts.len() != 2 {
            return Err(
                "SET requires: SET variable value"
                    .to_string()
            );
        }

        let name =
            parts[0].to_string();

        let value =
            parts[1]
                .parse::<i64>()
                .map_err(|_| {
                    "SET value must be an integer"
                        .to_string()
                })?;

        return Ok(Some(
            Instruction::Set(
                name,
                value
            )
        ));
    }

    // ---------------------------------------------------------
    // ADD variable value
    // ---------------------------------------------------------

    if line.len() >= 3
        && line[..3].eq_ignore_ascii_case("ADD")
    {
        let parts: Vec<&str> =
            line[3..]
                .split_whitespace()
                .collect();

        if parts.len() != 2 {
            return Err(
                "ADD requires: ADD variable value"
                    .to_string()
            );
        }

        let name =
            parts[0].to_string();

        let value =
            parts[1]
                .parse::<i64>()
                .map_err(|_| {
                    "ADD value must be an integer"
                        .to_string()
                })?;

        return Ok(Some(
            Instruction::Add(
                name,
                value
            )
        ));
    }

    // ---------------------------------------------------------
    // SUB variable value
    // ---------------------------------------------------------

    if line.len() >= 3
        && line[..3].eq_ignore_ascii_case("SUB")
    {
        let parts: Vec<&str> =
            line[3..]
                .split_whitespace()
                .collect();

        if parts.len() != 2 {
            return Err(
                "SUB requires: SUB variable value"
                    .to_string()
            );
        }

        let name =
            parts[0].to_string();

        let value =
            parts[1]
                .parse::<i64>()
                .map_err(|_| {
                    "SUB value must be an integer"
                        .to_string()
                })?;

        return Ok(Some(
            Instruction::Sub(
                name,
                value
            )
        ));
    }

    // ---------------------------------------------------------
    // INC variable
    // ---------------------------------------------------------

    if line.len() >= 3
        && line[..3].eq_ignore_ascii_case("INC")
    {
        let name =
            line[3..].trim();

        if name.is_empty() {
            return Err(
                "INC requires a variable name"
                    .to_string()
            );
        }

        return Ok(Some(
            Instruction::Inc(
                name.to_string()
            )
        ));
    }

    // ---------------------------------------------------------
    // DEC variable
    // ---------------------------------------------------------

    if line.len() >= 3
        && line[..3].eq_ignore_ascii_case("DEC")
    {
        let name =
            line[3..].trim();

        if name.is_empty() {
            return Err(
                "DEC requires a variable name"
                    .to_string()
            );
        }

        return Ok(Some(
            Instruction::Dec(
                name.to_string()
            )
        ));
    }

    // ---------------------------------------------------------
    // GOTO label
    // ---------------------------------------------------------

    if line.len() >= 4
        && line[..4].eq_ignore_ascii_case("GOTO")
    {
        let label =
            line[4..].trim();

        if label.is_empty() {
            return Err(
                "GOTO requires a label"
                    .to_string()
            );
        }

        return Ok(Some(
            Instruction::Goto(
                label.to_string()
            )
        ));
    }

    // ---------------------------------------------------------
    // IF variable operator value GOTO label
    // ---------------------------------------------------------

    if line.len() >= 2
        && line[..2].eq_ignore_ascii_case("IF")
    {
        let parts: Vec<&str> =
            line[2..]
                .split_whitespace()
                .collect();

        if parts.len() != 5 {
            return Err(
                "IF requires: IF variable operator value GOTO label"
                    .to_string()
            );
        }

        let variable =
            parts[0].to_string();

        let comparison =
            parse_comparison(parts[1])
                .ok_or_else(|| {
                    "Unknown comparison operator"
                        .to_string()
                })?;

        let value =
            parts[2]
                .parse::<i64>()
                .map_err(|_| {
                    "IF comparison value must be an integer"
                        .to_string()
                })?;

        if !parts[3]
            .eq_ignore_ascii_case("GOTO")
        {
            return Err(
                "IF requires GOTO"
                    .to_string()
            );
        }

        let label =
            parts[4].to_string();

        return Ok(Some(
            Instruction::If {
                variable,
                comparison,
                value,
                label,
            }
        ));
    }

    Err(format!(
        "Unknown command: {}",
        line
    ))
}

fn execute_script(
    script_path: &Path,
    runtime_log: &mut String,
) {
    let script_name =
        script_path
            .file_name()
            .map(|name| {
                name.to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| {
                "<unknown>".to_string()
            });

    runtime_log.push_str(&format!(
        "\n========== {} ==========\n",
        script_name
    ));

    let contents =
        match fs::read_to_string(script_path) {
            Ok(contents) => contents,

            Err(error) => {
                runtime_log.push_str(
                    &format!(
                        "[ERROR] Could not read script: {}\n",
                        error
                    )
                );

                return;
            }
        };

    let lines: Vec<&str> =
        contents.lines().collect();

    let mut labels =
        HashMap::<String, usize>::new();

    // ---------------------------------------------------------
    // First pass: collect labels
    // ---------------------------------------------------------

    for (index, line) in lines.iter().enumerate() {
        let trimmed =
            line.trim();

        if trimmed.starts_with(':') {
            let label =
                trimmed[1..].trim();

            if !label.is_empty() {
                labels.insert(
                    label.to_string(),
                    index,
                );
            }
        }
    }

    runtime_log.push_str(&format!(
        "Found {} label(s)\n",
        labels.len()
    ));

    let mut instructions =
        Vec::<(usize, Instruction)>::new();

    // ---------------------------------------------------------
    // Second pass: parse instructions
    // ---------------------------------------------------------

    for (line_number, line) in lines.iter().enumerate() {
        match parse_instruction(line) {
            Ok(Some(instruction)) => {
                instructions.push((
                    line_number,
                    instruction,
                ));
            }

            Ok(None) => {}

            Err(error) => {
                runtime_log.push_str(
                    &format!(
                        "[ERROR] Line {}: {}\n",
                        line_number + 1,
                        error
                    )
                );
            }
        }
    }

    let mut runtime =
        ScriptRuntime::new();

    let mut pc: usize = 0;

    let mut steps: u64 = 0;

    const MAX_STEPS: u64 = 100_000;

    while pc < instructions.len() {
        steps += 1;

        if steps > MAX_STEPS {
            runtime_log.push_str(
                "[ERROR] Maximum instruction limit reached.\n"
            );

            break;
        }

        let (source_line, instruction) =
            &instructions[pc];

        match instruction {
            Instruction::Goto(label) => {
                runtime.execute(
                    instruction,
                    runtime_log,
                );

                match labels.get(label) {
                    Some(target_line) => {
                        if let Some(target_pc) =
                            instructions
                                .iter()
                                .position(
                                    |(line, _)| {
                                        line == target_line
                                    },
                                )
                        {
                            pc = target_pc;
                            continue;
                        }

                        runtime_log.push_str(
                            &format!(
                                "[ERROR] GOTO label not executable: {}\n",
                                label
                            )
                        );
                    }

                    None => {
                        runtime_log.push_str(
                            &format!(
                                "[ERROR] Unknown label: {}\n",
                                label
                            )
                        );
                    }
                }
            }

            Instruction::If {
                variable,
                comparison,
                value,
                label,
            } => {
                let condition =
                    runtime.compare(
                        variable,
                        comparison,
                        *value,
                    );

                runtime_log.push_str(
                    &format!(
                        "[IF] {} -> {}\n",
                        variable,
                        condition
                    )
                );

                if condition {
                    match labels.get(label) {
                        Some(target_line) => {
                            if let Some(target_pc) =
                                instructions
                                    .iter()
                                    .position(
                                        |(line, _)| {
                                            line == target_line
                                        },
                                    )
                            {
                                runtime_log.push_str(
                                    &format!(
                                        "[IF] jumping to {}\n",
                                        label
                                    )
                                );

                                pc = target_pc;
                                continue;
                            }

                            runtime_log.push_str(
                                &format!(
                                    "[ERROR] IF label not executable: {}\n",
                                    label
                                )
                            );
                        }

                        None => {
                            runtime_log.push_str(
                                &format!(
                                    "[ERROR] Unknown label: {}\n",
                                    label
                                )
                            );
                        }
                    }
                }
            }

            _ => {
                runtime.execute(
                    instruction,
                    runtime_log,
                );
            }
        }

        runtime_log.push_str(
            &format!(
                "[LINE] {}\n",
                source_line + 1
            )
        );

        pc += 1;
    }

    runtime_log.push_str(
        "========== SCRIPT END ==========\n"
    );
}

fn load_scripts(root: &Path) {
    let scripts_dir =
        root.join("scripts");

    let runtime_log_path =
        root.join("logs")
            .join("runtime.log");

    let mut runtime_log =
        String::from(
            "CLEO Custom Runtime\n\
====================\n\
Version: 0.5.0\n\
Script engine: enabled\n\
Control flow: enabled\n\
Variables: enabled\n\
WAIT: enabled\n\
IF/GOTO: enabled\n\n",
        );

    let entries =
        match fs::read_dir(&scripts_dir) {
            Ok(entries) => entries,

            Err(error) => {
                runtime_log.push_str(
                    &format!(
                        "[ERROR] Could not read scripts directory: {}\n",
                        error
                    )
                );

                write_file(
                    &runtime_log_path,
                    runtime_log.as_bytes(),
                );

                return;
            }
        };

    let mut scripts =
        Vec::new();

    for entry in entries.flatten() {
        let path =
            entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(extension) =
            path.extension()
        else {
            continue;
        };

        if extension
            .to_string_lossy()
            .eq_ignore_ascii_case("cs")
        {
            scripts.push(path);
        }
    }

    scripts.sort();

    runtime_log.push_str(
        &format!(
            "Found {} script(s)\n",
            scripts.len()
        )
    );

    for script in scripts {
        execute_script(
            &script,
            &mut runtime_log,
        );
    }

    write_file(
        &runtime_log_path,
        runtime_log.as_bytes(),
    );
}

pub fn cleo_init() {
    let Some(root) =
        cleo_root()
    else {
        return;
    };

    create_directory(&root);

    create_directory(
        &root.join("scripts")
    );

    create_directory(
        &root.join("config")
    );

    create_directory(
        &root.join("plugins")
    );

    create_directory(
        &root.join("logs")
    );

    write_file(
        &root.join("logs")
            .join("startup.log"),
        b"CLEO custom runtime initialized successfully.\n",
    );

    write_file(
        &root.join("config")
            .join("runtime.txt"),
        b"CLEO Custom Runtime\n\
Version: 0.5.0\n\
Platform: iOS arm64\n\
Loader: LiveContainer\n\
Script discovery: enabled\n\
Script execution: enabled\n\
Variables: enabled\n\
WAIT: enabled\n\
IF/GOTO: enabled\n\
Labels: enabled\n",
    );

    load_scripts(&root);

    write_file(
        &root.join(
            "LIVE_CONTAINER_DIAGNOSTIC.txt"
        ),
        b"CLEO LiveContainer diagnostic loaded successfully.\n",
    );
}

#[no_mangle]
pub extern "C" fn cleo_diagnostic_init() {
    cleo_init();
}

#[used]
#[cfg_attr(
    target_os = "ios",
    link_section = "__DATA,__mod_init_func"
)]
static INIT: extern "C" fn() = {
    extern "C" fn init() {
        cleo_init();
    }

    init
};