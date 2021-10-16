pub fn filter_blackbox_directives(t: &str) -> String {
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
