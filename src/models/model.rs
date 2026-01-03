use std::path::PathBuf;

pub struct OutputParams {
    path: PathBuf,
}

impl OutputParams {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
