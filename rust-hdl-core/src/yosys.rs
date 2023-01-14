use crate::prelude::*;
use std::env::temp_dir;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::{Error, Write};
use std::process::Command;

#[derive(Debug)]
pub enum SynthError {
    SynthesisFailed { stdout: String, stderr: String },
    LatchingWriteToSignal(Vec<String>),
    ImplicitlyDeclared(Vec<String>),
    DuplicateModule(Vec<String>),
    IOError(std::io::Error),
    WireHasNoDriver(Vec<String>),
    MissingModule(Vec<String>),
}

impl From<std::io::Error> for SynthError {
    fn from(x: Error) -> Self {
        SynthError::IOError(x)
    }
}

pub fn yosys_validate(prefix: &str, translation: &str) -> Result<(), SynthError> {
    let dir = temp_dir().as_path().join(prefix);
    let _ = remove_dir_all(&dir);
    let _ = create_dir_all(&dir);
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
        let mut debug = File::create(dir.join("yosys.stdout"))?;
        write!(debug, "{}", stdout).unwrap();
        write!(debug, "{}", stderr).unwrap();
        let mut dump = File::create(dir.join("yosys.v"))?;
        write!(dump, "{}", translation).unwrap();
    }
    fn capture(stdout: &str, reg_exp: &str) -> Vec<String> {
        let regex = regex::Regex::new(reg_exp).unwrap();
        let mut signal_name = vec![];
        if regex.is_match(&stdout) {
            for capture in regex.captures(&stdout).unwrap().iter() {
                signal_name.push(capture.unwrap().as_str().to_string());
            }
        }
        signal_name
    }
    if stdout.contains("Re-definition of") {
        return Err(SynthError::DuplicateModule(capture(
            &stdout,
            r#"Re-definition of module (\S*)"#,
        )));
    }
    if stdout.contains("implicitly declared.") {
        return Err(SynthError::ImplicitlyDeclared(capture(
            &stdout,
            r#"Identifier (\S*) is implicitly declared"#,
        )));
    }
    if stdout.contains("Latch inferred for") {
        return Err(SynthError::LatchingWriteToSignal(capture(
            &stdout,
            r#"Latch inferred for signal (\S*)"#,
        )));
    }
    if stdout.contains("is used but has no driver") {
        return Err(SynthError::WireHasNoDriver(capture(
            &stdout,
            r#"Wire (\S*) .*? is used but has no driver."#,
        )));
    }
    if stderr.contains("is not part of the design") {
        return Err(SynthError::MissingModule(capture(
            &stderr,
            r#"Module (\S*) .*? is not part of the design"#,
        )));
    }
    if !stdout.contains("End of script.") {
        return Err(SynthError::SynthesisFailed { stdout, stderr });
    }
    Ok(())
}

#[derive(LogicBlock)]
pub struct TopWrap<U: Block> {
    pub uut: U,
}

impl<U: Block> TopWrap<U> {
    pub fn new(uut: U) -> Self {
        Self { uut }
    }
}

impl<U: Block> Logic for TopWrap<U> {
    fn update(&mut self) {}
}
