use nih_plug::prelude::*;
use std::sync::Arc;
use atomic_float::AtomicF32; // 追加
use nih_plug_iced::IcedState; // 追加

mod editor; // 追加

/// 完全に無音に切り替えた後、ピークメーターが12dB減衰するまでの時間。
const PEAK_METER_DECAY_MS: f64 = 150.0; // 追加

struct Claptest {
    params: Arc<ClaptestParams>,

    /// ピークメーターの正規化用
    peak_meter_decay_weight: f32, // 追加
    /// ピークメーターの現在のデータ
    peak_meter: Arc<AtomicF32>, // 追加
}

#[derive(Params)]
struct ClaptestParams {
    /// エディターの状態
    #[persist = "editor-state"] // 追加
    editor_state: Arc<IcedState>, // 追加

    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for Claptest {
    fn default() -> Self {
        Self {
            params: Arc::new(ClaptestParams::default()),

            peak_meter_decay_weight: 1.0, // 追加
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)), // 追加
        }
    }
}

impl Default for ClaptestParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(), // 追加

            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

impl Plugin for Claptest {
    const NAME: &'static str = "Claptest";
    const VENDOR: &'static str = "Saisana299";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "your@email.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    /// AUDIO_IO_LAYOUTSを変更
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

    /// 以下は削除
    // const MIDI_INPUT: MidiConfig = MidiConfig::None;
    // const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    /// 以下を追加
    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.peak_meter.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig, // アンダーバーを消す
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // 以下を追加
        self.peak_meter_decay_weight = 0.25f64
            .powf((buffer_config.sample_rate as f64 * PEAK_METER_DECAY_MS / 1000.0).recip())
            as f32;

        true
    }

    /// 以下を削除
    // fn reset(&mut self) {
    // }

    /// processの内容を編集
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let mut amplitude = 0.0;
            let num_samples = channel_samples.len();

            let gain = self.params.gain.smoothed.next();
            for sample in channel_samples {
                *sample *= gain;
                amplitude += *sample;
            }

            // GUIが表示されている時のみGUIの計算を行う
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

impl ClapPlugin for Claptest {
    const CLAP_ID: &'static str = "com.your-domain.claptest";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A short description of your plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

     const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

nih_export_clap!(Claptest);
