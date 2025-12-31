use gpui::{
    AbsoluteLength, BorderStyle, Bounds, Corners, DefiniteLength, Element, IntoElement, LayoutId,
    Length, Path, Pixels, Size, Style, point, px, quad, relative, rgb,
};

pub struct Timeline {
    percent: f32,
}

impl Timeline {
    pub fn new(percent: f32) -> Self {
        Self { percent: percent }
    }

    fn indicator_x(&self, b: gpui::Bounds<gpui::Pixels>) -> Pixels {
        (b.size.width * self.percent).round()
    }
}

impl Element for Timeline {
    type RequestLayoutState = LayoutId;

    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();

        style.size.width = relative(1.0).into();
        style.size.height =
            Length::Definite(DefiniteLength::Absolute(AbsoluteLength::Pixels(px(30.))));

        let layout_id = window.request_layout(style, None, cx);
        (layout_id, layout_id)
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut gpui::Window,
        _: &mut gpui::App,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        _: &mut gpui::App,
    ) {
        // timeline base
        window.paint_quad(quad(
            Bounds {
                origin: bounds.origin,
                size: Size {
                    width: bounds.size.width,
                    height: px(10.),
                },
            },
            Corners::default(),
            rgb(0x65acd7),
            px(0.),
            gpui::white(),
            BorderStyle::default(),
        ));

        // triangle size
        let scale = window.scale_factor();
        let head_size = px(5.0 / scale);

        let width = px(1.0 / scale);
        let height = px(16.);
        let x = self.indicator_x(bounds);
        let y = bounds.origin.y - px(14.);
        let color = gpui::white();
        let mut path = Path::new(bounds.origin);

        // paint triangle
        path.move_to(point(x - head_size, y)); // left top
        path.line_to(point(x + head_size, y)); // right top
        path.line_to(point(x, y + head_size)); // bottom corner
        path.line_to(point(x - head_size, y)); // back to start
        window.paint_path(path, color);

        // paint indicator line
        window.paint_quad(quad(
            Bounds {
                origin: point(x - width / 2.0, bounds.origin.y - px(3.)),
                size: Size {
                    width: width,
                    height: height,
                },
            },
            Corners::default(),
            color,
            px(0.),
            color,
            BorderStyle::default(),
        ));
    }
}

impl IntoElement for Timeline {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}
