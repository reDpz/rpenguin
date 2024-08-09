#[macro_export]
// i gotta learn more of this shit it's unbelievably good
macro_rules! crate_path {
    ($path:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/", $path)
    };
}
