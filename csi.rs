use byteorder::{LittleEndian, ReadBytesExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{self, Cursor, Read};

#[derive(Debug, Deserialize)]
enum ParamType { Integer, String, Real, Model, Pointer, End, Buffer, Any }

#[derive(Debug, Deserialize, Clone, Copy)]
enum Location { Immediate, Local, Global }

#[derive(Debug, Deserialize)]
struct Param {
    param_type: ParamType,
    location: Location,
    is_variadic: bool,
    is_output: bool,
}

#[derive(Debug, Deserialize)]
struct Command {
    opcode: u16,
    name: String,
    returns: bool,
    params: Vec<Param>,
}

#[derive(Debug)]
enum Value {
    Integer(i64),
    Real(f32),
    String(String),
    Variable { id: u16, global: bool },
    Array,
    Pointer(i64),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(v) => write!(f, "{v}"),
            Value::Real(v) => write!(f, "{v}f"),
            Value::String(v) => write!(f, "\"{v}\""),
            Value::Variable { id, global } => {
                write!(f, "{}_{id:#x}", if *global { "global" } else { "local" })
            }
            Value::Array => write!(f, "<array>"),
            Value::Pointer(v) => write!(f, "ptr({v:#x})"),
        }
    }
}

fn read_value(reader: &mut Cursor<&[u8]>) -> io::Result<Value> {
    let id = reader.read_u8()?;
    Ok(match id {
        0x01 => Value::Integer(reader.read_i32::<LittleEndian>()? as i64),
        0x02 | 0x0a | 0x10 => Value::Variable { id: reader.read_u16::<LittleEndian>()?, global: true },
        0x03 | 0x0b | 0x11 => Value::Variable { id: reader.read_u16::<LittleEndian>()?, global: false },
        0x04 => Value::Integer(reader.read_i8()? as i64),
        0x05 => Value::Integer(reader.read_i16::<LittleEndian>()? as i64),
        0x06 => Value::Real(reader.read_f32::<LittleEndian>()?),
        0x07 | 0x08 | 0x0c | 0x0d | 0x12 | 0x13 => {
            let mut buf = [0u8; 6]; reader.read_exact(&mut buf)?; Value::Array
        }
        0x09 => {
            let mut buf = [0u8; 8]; reader.read_exact(&mut buf)?;
            Value::String(String::from_utf8_lossy(&buf).trim_end_matches('\0').to_string())
        }
        0x0e => {
            let len = reader.read_u8()? as usize;
            let mut buf = vec![0u8; len]; reader.read_exact(&mut buf)?;
            Value::String(String::from_utf8_lossy(&buf).trim_end_matches('\0').to_string())
        }
        0x0f => {
            let mut buf = [0u8; 16]; reader.read_exact(&mut buf)?;
            Value::String(String::from_utf8_lossy(&buf).trim_end_matches('\0').to_string())
        }
        other => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown parameter type 0x{other:02x}"))),
    })
}

fn read_commands() -> Result<HashMap<u16, Command>, Box<bincode::ErrorKind>> {
    bincode::deserialize(include_bytes!("../commands.bin"))
}

pub struct ScriptInfo {
    pub kind: &'static str,
    pub size: usize,
    pub instructions: usize,
}

pub fn inspect(bytes: &[u8], kind: &'static str, log: &mut dyn FnMut(String)) -> Result<ScriptInfo, String> {
    let commands = read_commands().map_err(|e| format!("commands.bin: {e}"))?;
    let mut reader = Cursor::new(bytes);
    let mut count = 0usize;

    while (reader.position() as usize) < bytes.len() {
        let offset = reader.position();
        let raw = reader.read_u16::<LittleEndian>().map_err(|e| format!("offset {offset}: {e}"))?;
        let inverted = raw & 0x8000 != 0;
        let opcode = raw & 0x7fff;

        let command = commands.get(&opcode).ok_or_else(|| {
            format!("unknown opcode 0x{opcode:04x} at offset 0x{offset:x}")
        })?;

        let mut args = Vec::new();
        for param in &command.params {
            if matches!(param.param_type, ParamType::End) { break; }
            let mut value = read_value(&mut reader)
                .map_err(|e| format!("opcode 0x{opcode:04x} ({}) at 0x{offset:x}: {e}", command.name))?;

            if matches!(param.param_type, ParamType::Pointer) {
                if let Value::Integer(v) = value { value = Value::Pointer(v); }
            }
            args.push(value);
        }

        log(format!(
            "[CSI] 0x{offset:04x}: {}{}{}",
            if inverted { "NOT " } else { "" },
            command.name,
            if args.is_empty() { String::new() } else { format!(" {}", args.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")) }
        ));
        count += 1;

        if count > 100_000 { return Err("instruction safety limit exceeded".into()); }
    }

    Ok(ScriptInfo { kind, size: bytes.len(), instructions: count })
}
