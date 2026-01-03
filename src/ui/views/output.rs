use gpui::{Entity, ParentElement, Render, Styled, div};

use crate::models::model::OutputParams;

pub struct OutputView {
    params: Entity<Option<OutputParams>>,
}

impl OutputView {
    pub fn new(params: Entity<Option<OutputParams>>) -> Self {
        Self { params }
    }
}

impl Render for OutputView {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .child("Output View")
    }
}
