pub mod consent;
pub mod url_parser;

pub fn default_data_dir() -> std::path::PathBuf {
    ygg_core::paths::data_dir().expect("could not resolve Yggdrasil data dir")
}
