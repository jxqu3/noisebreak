use atomic_float::AtomicF32;
use lp::LowPassFilter;
use nih_plug::prelude::*;
use nih_plug_iced::IcedState;


use nih_plug::params::Params;
use rand::Rng;
use std::sync::{Arc, Mutex};
mod editor;
mod lp;
mod theme;
mod widgets;

/// The time it takes for the peak meter to decay by 12 dB after switching to complete silence.
const PEAK_METER_DECAY_MS: f64 = 150.0;

/// This is mostly identical to the gain example, minus some fluff, and with a GUI.
struct Noisebreak {
    params: Arc<NoisebreakParams>,
    current_sample: u8,

    /// Needed to normalize the peak meter's response based on the sample rate.
    peak_meter_decay_weight: f32,
    /// The current data for the peak meter. This is stored as an [`Arc`] so we can share it between
    /// the GUI and the audio processing parts. If you have more state to share, then it's a good
    /// idea to put all of that in a struct behind a single `Arc`.
    ///
    /// This is stored as voltage gain.
    peak_meter: Arc<AtomicF32>,

    sample_rate: f32,

    pub lp_filter: Arc<Mutex<LowPassFilter>>,
}

#[derive(Params)]
struct NoisebreakParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    
    
    #[persist = "editor-state"]
    editor_state: Arc<IcedState>,

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "cutoff"]
    pub cutoff: FloatParam,

    #[id = "interval"]
    pub interval: IntParam,

    #[id = "hp"]
    pub hp: BoolParam,

    #[id = "q"]
    pub q: FloatParam,
}

impl Default for Noisebreak {
    fn default() -> Self {
        Self {
            params: Arc::new(NoisebreakParams::default()),
            current_sample: 0,
            peak_meter_decay_weight: 1.0,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
            sample_rate: 44100.0,
            lp_filter: Arc::new(Mutex::new(LowPassFilter::new(500.0, 44100.0, 0.71))),
        }
    }
}

impl Default for NoisebreakParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            // Gain
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-70.0),
                    max: util::db_to_gain(0.0),
                    factor: FloatRange::gain_skew_factor(-70.0, 0.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            // Interval
            interval: IntParam::new("Interval", 0, IntRange::Linear { min: 0, max: 50 })
                .with_smoother(SmoothingStyle::Linear(1.)),

            // Cutoff
            cutoff: FloatParam::new(
                "Cutoff",
                500.0,
                FloatRange::Skewed {
                    min: 30.0,
                    max: 20_000.0,
                    factor: 0.4,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

            // HP
            hp: BoolParam::new("Interval", false),

            q: FloatParam::new("Q", 0.71, FloatRange::Linear { min: 0.1, max: 2.0 }),
        }
    }
}

impl Plugin for Noisebreak {
    const NAME: &'static str = "NOISEBREAK";
    const VENDOR: &'static str = "JXQU3";
    const URL: &'static str = "https://www.youtube.com/@JXQU3";
    const EMAIL: &'static str = "info@example.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.peak_meter.clone(),
            self.params.editor_state.clone(),
            self.lp_filter.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // After `PEAK_METER_DECAY_MS` milliseconds of pure silence, the peak meter's value should
        // have dropped by 12 dB
        self.peak_meter_decay_weight = 0.25f64
            .powf((buffer_config.sample_rate as f64 * PEAK_METER_DECAY_MS / 1000.0).recip())
            as f32;

        self.sample_rate = buffer_config.sample_rate as f32;
        self.lp_filter.lock().unwrap().set_sample_rate(self.sample_rate);

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let mut amplitude = 0.0;
            let mut rng = rand::thread_rng();
            let gain = self.params.gain.smoothed.next();
            

            if self.current_sample >= self.params.interval.smoothed.next() as u8 {
                self.current_sample = 0;
            }
            let noise: f32 = if self.current_sample == 0 {
                rng.gen_range(-1f32..1f32) * gain
            } else {
                0f32
            };
            self.current_sample += 1;
            let num_samples = channel_samples.len();
            let mut filter = self.lp_filter.lock().unwrap();

            for sample in channel_samples {
                let filtered_noise = if self.params.hp.value() {
                    noise + filter.process(-noise)
                } else {
                    filter.process(noise)
                };

                *sample += filtered_noise;
                amplitude += filtered_noise;
            }

            // To save resources, a plugin can (and probably should!) only perform expensive
            // calculations that are only displayed on the GUI while the GUI is open
            if self.params.editor_state.is_open() {
                amplitude = (amplitude / num_samples as f32).abs();
                let current_peak_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);
                let new_peak_meter = if amplitude > current_peak_meter {
                    amplitude
                } else {
                    current_peak_meter * self.peak_meter_decay_weight
                        + amplitude * (1.0 - self.peak_meter_decay_weight)
                };

               

                self.peak_meter
                    .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed)
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Noisebreak {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.gain-gui-iced";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A smoothed gain parameter example plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for Noisebreak {
    const VST3_CLASS_ID: [u8; 16] = *b"GainGuiIcedAaAAa";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_vst3!(Noisebreak);
