#[derive(Debug, Clone)]
pub struct Config {
    pub storage_path: String,
    pub media_path: String,
    pub cache_path: String,
}


impl Config {
    pub fn new(storage_path: String, media_path: String) -> Self {
        Self {
            storage_path: storage_path.clone(),
            media_path,
            cache_path: Self::cache_path(storage_path),
        }
    }

    fn cache_path(storage_path: String) -> String {
        format!("{}/{}", storage_path.trim_end_matches("/"), ".cache/")
    }
}