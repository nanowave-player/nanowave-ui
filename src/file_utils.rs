use std::path::Path;

pub fn filename_stem(path: &Path) -> Option<&str> {
    path.file_stem()?.to_str()
}