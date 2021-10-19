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
