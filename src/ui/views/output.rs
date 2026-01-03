use gpui::{AppContext, Entity, ParentElement, Render, Styled, div};
use gpui_component::{
    Sizable, StyledExt,
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    input::{Input, InputState},
    label::Label,
};

use crate::{models::model::OutputParams, ui::output::output::output};

pub struct OutputView {
    params: Entity<OutputParams>,
    input: Entity<InputState>,
}

impl OutputView {
    pub fn new(
        window: &mut gpui::Window,
        cx: &mut gpui::App,
        params: Entity<OutputParams>,
    ) -> Self {
        Self {
            params,
            input: cx.new(|cx| InputState::new(window, cx).default_value("./output.mp4")),
        }
    }

    pub fn run_output(&self, cx: &mut gpui::App) {
        let param = self.params.read(cx);
        let Some(path) = param.path.as_ref() else {
            return;
        };
        let Some(v_ix) = param.video_stream_ix else {
            return;
        };
        let Some(range) = param.selected_range else {
            return;
        };
        output(path, v_ix, range);
        // else {
        //     println!("DEBUG: error when output: None params")
        // }
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
            .v_flex()
            .p_3()
            .justify_between()
            .child(
                div()
                    .flex()
                    .v_flex()
                    .gap_3()
                    .child(
                        div().w_full().child(Label::new("Output Path")).child(
                            div()
                                .w_full()
                                .flex()
                                .h_flex()
                                .child(Input::new(&self.input))
                                .child(Button::new("select").ghost().label("...")),
                        ),
                    )
                    .child(
                        div()
                            .w_full()
                            // .child(Label::new("Output Path"))
                            .child(
                                Checkbox::new("checkbox")
                                    .label("Copy Stream")
                                    .checked(true)
                                    .on_click(|_, _, _| {}),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("cancel")
                            .small()
                            .label("Cancel")
                            .on_click(|_, w, _| {
                                w.remove_window();
                            }),
                    )
                    .child(
                        Button::new("output")
                            .small()
                            .primary()
                            .label("Output")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.run_output(cx);
                            })),
                    ),
            )
    }
}
