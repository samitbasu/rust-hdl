use std::env::temp_dir;
use std::fs::{create_dir, remove_dir_all, File};
use std::io::{Error, Write};
use std::process::Command;

#[derive(Debug)]
pub enum SynthError {
    SynthesisFailed { stdout: String, stderr: String },
    LatchingWriteToSignal(Vec<String>),
    IOError(std::io::Error),
}

impl From<std::io::Error> for SynthError {
    fn from(x: Error) -> Self {
        SynthError::IOError(x)
    }
}

pub fn yosys_validate(prefix: &str, translation: &str) -> Result<(), SynthError> {
    let dir = temp_dir().as_path().join(prefix);
    let _ = remove_dir_all(&dir);
    let _ = create_dir(&dir);
    let mut v_file = File::create(dir.clone().join("top.v")).unwrap();
    write!(v_file, "{}", translation).unwrap();
    let output = Command::new("yosys")
        .current_dir(dir.clone())
        .arg(format!(
            "-p read -vlog95 top.v; hierarchy -check -top top; proc",
        ))
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    {
        let mut debug = File::create("yosys.stdout")?;
        write!(debug, "{}", stdout).unwrap();
        write!(debug, "{}", stderr).unwrap();
        let mut dump = File::create("yosys.v")?;
        write!(dump, "{}", translation).unwrap();
    }
    if stdout.contains("implicitly declared.") {
        return Err(SynthError::SynthesisFailed { stdout, stderr });
    }
    if stdout.contains("Latch inferred for") {
        let regex = regex::Regex::new(r#"Latch inferred for signal (\S*)"#).unwrap();
        let mut signal_name = vec![];
        if regex.is_match(&stdout) {
            for capture in regex.captures(&stdout).unwrap().iter() {
                signal_name.push(capture.unwrap().as_str().to_string());
            }
        }
        return Err(SynthError::LatchingWriteToSignal(signal_name));
    }
    if !stdout.contains("End of script.") {
        return Err(SynthError::SynthesisFailed { stdout, stderr });
    }
    Ok(())
}
