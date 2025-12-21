use crate::ui::utils;

#[derive(Debug)]
pub struct PlayerSize {
    orignal_size: (u32, u32),
    view_size: (u32, u32),
    output_size: (u32, u32),
}

impl PlayerSize {
    pub fn new() -> Self {
        Self {
            orignal_size: (0, 0),
            view_size: (0, 0),
            output_size: (1, 1),
        }
    }
    pub fn set_size(&mut self, orignal: Option<(u32, u32)>, view: Option<(u32, u32)>) {
        if let Some(o) = orignal {
            self.orignal_size = o;
        }
        if let Some(v) = view {
            self.view_size = v;
        }
        if let Some(out_size) = utils::calc_output_size(self.orignal_size, self.view_size) {
            self.output_size = out_size;
        }
    }

    pub fn set_orignal(&mut self, size: (u32, u32)) {
        self.set_size(Some(size), None);
    }
    pub fn set_view(&mut self, size: (u32, u32)) {
        self.set_size(None, Some(size));
    }

    pub fn orignal_size(&self) -> (u32, u32) {
        self.orignal_size
    }
    pub fn view_size(&self) -> (u32, u32) {
        self.view_size
    }
    pub fn output_size(&self) -> (u32, u32) {
        self.output_size
    }
}
