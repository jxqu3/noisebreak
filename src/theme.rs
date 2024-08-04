use nih_plug_iced::*;

pub const FG_COLOR : Color = Color {
    r: 0.15,
    g: 0.15,
    b: 0.15,
    a: 1.0,
};

pub const BG_COLOR : Color = Color {
    r: 0.90,
    g: 0.90,
    b: 0.90,
    a: 1.0,
};
pub const FONT : Font = assets::NOTO_SANS_LIGHT;

pub fn param_label(name: &str) -> Text {
    Text::new(name)
        .color(FG_COLOR)
        .font(FONT)
        .height(20.into())
        .width(Length::Fill)
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Center)
}

pub fn title(name: &str) -> Text {
    Text::new(name)
        .color(FG_COLOR)
        .font(assets::NOTO_SANS_BOLD)
        .size(40)
        .height(50.into())
        .width(Length::Fill)
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Bottom)
}

#[macro_export]
macro_rules! param_slider {
    ($name: expr, $state: expr, $param: expr) => {
       Column::new()
        .align_items(Alignment::Center)
        .push(
            param_label($name)
        )
        .push(
            nih_widgets::ParamSlider::new(&mut $state, &$param)
            .map(Message::ParamUpdate)
        )
    };
}
