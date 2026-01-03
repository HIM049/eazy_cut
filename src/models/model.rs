use std::path::PathBuf;

pub struct OutputParams {
    pub path: Option<PathBuf>,
    pub video_stream_ix: Option<usize>,
    pub audio_stream_ix: Option<usize>,
    pub selected_range: Option<(f32, f32)>,
}

impl OutputParams {
    pub fn default() -> Self {
        Self {
            path: None,
            video_stream_ix: None,
            audio_stream_ix: None,
            selected_range: None,
        }
    }
}
