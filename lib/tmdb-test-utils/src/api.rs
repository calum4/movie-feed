pub mod misc;
pub mod v3;

#[inline]
pub(crate) fn file_path<'a>(path: &'a str, file_name: &'a str) -> String {
    const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
    format!("{MANIFEST_DIR}/response_files{path}/{file_name}")
}
