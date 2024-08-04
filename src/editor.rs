use crate::{
    lp::LowPassFilter, param_slider, theme::{self, param_label, title, FONT}, NoisebreakParams
};
use atomic_float::AtomicF32;
use std::sync::Mutex;
use nih_plug::prelude::{util, Editor, GuiContext};
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use std::sync::Arc;
use std::time::Duration;
use theme::BG_COLOR;

const WIDTH: u32 = 300;
const HEIGHT: u32 = 300;

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<IcedState> {
    IcedState::from_size(WIDTH, HEIGHT)
}

pub(crate) fn create(
    params: Arc<NoisebreakParams>,
    peak_meter: Arc<AtomicF32>,
    editor_state: Arc<IcedState>,
    lp_filter: Arc<Mutex<LowPassFilter>>,
) -> Option<Box<dyn Editor>> {
    create_iced_editor::<NoisebreakEditor>(editor_state, (params, peak_meter, lp_filter))
}

struct NoisebreakEditor {
    params: Arc<NoisebreakParams>,
    context: Arc<dyn GuiContext>,
    lp_filter: Arc<Mutex<LowPassFilter>>,

    peak_meter: Arc<AtomicF32>,

    gain_slider_state: nih_widgets::param_slider::State,
    interval_slider_state: nih_widgets::param_slider::State,
    cutoff_slider_state: nih_widgets::param_slider::State,
    resonance_state: nih_widgets::param_slider::State,
    peak_meter_state: nih_widgets::peak_meter::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    /// Update a parameter's value.
    ParamUpdate(nih_widgets::ParamMessage),

    // Update cutoff
    CutoffUpdate(nih_widgets::ParamMessage),
}

impl IcedEditor for NoisebreakEditor {
    type Executor = executor::Default;
    type Message = Message;
    type InitializationFlags = (Arc<NoisebreakParams>, Arc<AtomicF32>, Arc<Mutex<LowPassFilter>>);

    fn new(
        (params, peak_meter, lp_filter): Self::InitializationFlags,
        context: Arc<dyn GuiContext>,
    ) -> (Self, Command<Self::Message>) {
        let editor = NoisebreakEditor {
            params,
            context,
            lp_filter,

            peak_meter,

            gain_slider_state: Default::default(),
            interval_slider_state: Default::default(),
            cutoff_slider_state: Default::default(),
            resonance_state: Default::default(),
            peak_meter_state: Default::default(),
        };

        (editor, Command::none())
    }

    fn context(&self) -> &dyn GuiContext {
        self.context.as_ref()
    }

    fn update(
        &mut self,
        _window: &mut WindowQueue,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match message {
            Message::ParamUpdate(message) => self.handle_param_message(message),

            Message::CutoffUpdate(message) => {
                self.handle_param_message(message);

                self.lp_filter.lock().unwrap().set_cutoff_frequency(self.params.cutoff.value(), self.params.q.value());
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        Column::new()
            .align_items(Alignment::Center)
            // Title
            .push(title("NOISEBREAK"))
            // Noise Gain Slider
            .push(param_slider!(
                "gain [db]",
                self.gain_slider_state,
                self.params.gain
            ))
            // Interval Slider
            .push(param_slider!(
                "interval [samples]",
                self.interval_slider_state,
                self.params.interval
            ))
            // Cutoff Slider
            .push(
                Column::new()
                    .align_items(Alignment::Center)
                    .push(param_label("cutoff [hz]"))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.cutoff_slider_state,
                            &self.params.cutoff,
                        )
                        .map(Message::CutoffUpdate)
                    ),
            )
            .push(
                Column::new()
                    .align_items(Alignment::Center)
                    .push(param_label("resonance [q]"))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.resonance_state,
                            &self.params.q,
                        )
                        .map(Message::CutoffUpdate)
                    ),
            )
            .push(Space::with_height(10.into()))
            // Peak Meter
            .push(
                nih_widgets::PeakMeter::new(
                    &mut self.peak_meter_state,
                    util::gain_to_db(self.peak_meter.load(std::sync::atomic::Ordering::Relaxed)),
                )
                .font(FONT)
                .hold_time(Duration::from_millis(400)),
            )
            .into()
    }

    fn background_color(&self) -> nih_plug_iced::Color {
        BG_COLOR
    }
}
