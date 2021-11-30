#[macro_export]
macro_rules! target_path {
    ($name: expr) => {
        &std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("firmware")
            .join($name)
            .to_string_lossy()
            .to_string()
    };
}

#[macro_export]
macro_rules! vcd_path {
    ($name: expr) => {
        {let dest =
        &std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("sims");
        let _ = std::fs::create_dir(dest);
        dest
        .join($name)
        .to_string_lossy()
        .to_string()}
    }
}