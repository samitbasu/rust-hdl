use crate::ucf_gen::generate_ucf;
use rust_hdl_core::prelude::*;
use std::fs::{copy, create_dir, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn filter_blackbox_directives(t: &str) -> String {
    let mut in_black_box = false;
    let mut ret = vec![];
    for line in t.split("\n") {
        in_black_box = in_black_box || line.starts_with("(* blackbox *)");
        if !in_black_box {
            ret.push(line);
        }
        if line.starts_with("endmodule") {
            in_black_box = false;
        }
    }
    ret.join("\n")
}

#[test]
fn test_filter_bb_directives() {
    let p = r#"
blah
more code
goes here

(* blackbox *)
module my_famous_module(
    super_secret_arg1,
    super_secret_arg2,
    super_secret_arg3);
/* Comment */
endmodule

stuff
"#;
    let q = filter_blackbox_directives(p);
    println!("{}", q);
    assert!(!q.contains("blackbox"));
    assert!(!q.contains("module"));
    assert!(!q.contains("endmodule"));
    assert!(q.contains("more code"));
    assert!(q.contains("stuff"));
}

pub fn generate_bitstream_xem_6010<U: Block>(
    mut uut: U,
    prefix: &str,
    assets: &[&str],
    asset_dir: &str,
) {
    uut.connect_all();
    check_connected(&uut);
    let verilog_text = filter_blackbox_directives(&generate_verilog(&uut));
    let ucf_text = generate_ucf(&uut);
    let dir = PathBuf::from(prefix);
    let _ = remove_dir_all(&dir);
    let _ = create_dir(&dir);
    let mut v_file = File::create(dir.clone().join("top.v")).unwrap();
    write!(v_file, "{}", verilog_text).unwrap();
    let mut ucf_file = File::create(dir.clone().join("top.ucf")).unwrap();
    write!(ucf_file, "{}", ucf_text).unwrap();
    let asset_dir = PathBuf::from(asset_dir);
    for asset in assets {
        let source = asset_dir.join(asset);
        let dest = dir.clone().join(asset);
        copy(source, dest).unwrap();
    }
    let mut tcl_file = File::create(dir.clone().join("top.tcl")).unwrap();
    write!(
        tcl_file,
        "\
project new top.xise
project set family Spartan6
project set device xc6slx45
project set package fgg484
project set speed -3
xfile add top.v top.ucf {assets}
project set top top
process run \"Generate Programming File\" -force rerun_all
project close
",
        assets = assets.join(" ")
    )
    .unwrap();
    let output = Command::new("xtclsh")
        .current_dir(dir.clone())
        .arg("top.tcl")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let mut out_file = File::create(dir.clone().join("top.out")).unwrap();
    write!(out_file, "{}", stdout).unwrap();
    let mut err_file = File::create(dir.clone().join("top.err")).unwrap();
    write!(err_file, "{}", stderr).unwrap();
    assert!(stdout.contains(r#"Process "Generate Programming File" completed successfully"#));
    assert!(stdout.contains(r#"All constraints were met."#));
}

#[macro_export]
macro_rules! top_wrap {
    ($kind: ty, $name: ident) => {
        #[derive(LogicBlock, Default)]
        struct $name {
            uut: $kind
        }
        impl Logic for $name {
            fn update(&mut self) {}
        }
    }
}