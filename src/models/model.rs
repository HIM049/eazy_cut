use std::path::PathBuf;

use gpui::WindowHandle;
use gpui_component::Root;

pub struct WindowState {
    pub output_handle: Option<WindowHandle<Root>>,
    pub about_handle: Option<WindowHandle<Root>>,
}

impl WindowState {
    pub fn default() -> Self {
        Self {
            output_handle: None,
            about_handle: None,
        }
    }
}

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
