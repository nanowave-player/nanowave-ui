#[derive(Debug, Clone)]
pub struct Config {
    pub base_path: String,
    pub cache_path: String,
}


impl Config {
    pub fn new(base_path: String) -> Self {
        Self {
            base_path: base_path.clone(),
            cache_path: Self::cache_path(base_path.clone()),
        }
    }

    fn cache_path(base_path: String) -> String {
        format!("{}/{}", base_path.trim_end_matches("/"), ".cache/")
    }
}