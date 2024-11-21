use clack_extensions::audio_ports::*;
use clack_extensions::note_ports::*;
use clack_plugin::prelude::*;

pub struct ClapTest;

impl Plugin for ClapTest {
    type AudioProcessor<'a> = ClapTestAudioProcessor<'a>;
    type Shared<'a> = ClapTestShared;
    type MainThread<'a> = ClapTestMainThread<'a>;

    fn declare_extensions(builder: &mut PluginExtensions<Self>, _shared: Option<&ClapTestShared>) {
        builder
            .register::<PluginAudioPorts>();
    }
}

impl DefaultPluginFactory for ClapTest {
    fn get_descriptor() -> PluginDescriptor {
        use clack_plugin::plugin::features::*;

        PluginDescriptor::new("com.github.saisana299.clap-test", "Clap Test")
            .with_vendor("Saisana299")
            .with_features([AUDIO_EFFECT, STEREO])
            .with_version("1.0.0")
            .with_description("Test plugin for Clap")
    }

    fn new_shared(_host: HostSharedHandle) -> Result<Self::Shared<'_>, PluginError> {
        Ok(ClapTestShared {})
    }

    fn new_main_thread<'a>(
        _host: HostMainThreadHandle<'a>,
        shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(Self::MainThread { shared })
    }
}

pub struct ClapTestAudioProcessor<'a> {
    _shared: &'a ClapTestShared,
}

impl<'a> PluginAudioProcessor<'a, ClapTestShared, ClapTestMainThread<'a>>
    for ClapTestAudioProcessor<'a>
{
    fn activate(
        _host: HostAudioProcessorHandle<'a>,
        _main_thread: &mut ClapTestMainThread<'a>,
        _shared: &'a ClapTestShared,
        _audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self { _shared })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        let mut port_pair = audio
            .port_pair(0)
            .ok_or(PluginError::Message("No input/output ports found"))?;

        let mut output_channels = port_pair
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 input/output"))?;

        let mut channel_buffers = [None, None];

        for (pair, buf) in output_channels.iter_mut().zip(&mut channel_buffers) {
            *buf = match pair {
                ChannelPair::InputOnly(_) => None,
                ChannelPair::OutputOnly(_) => None,
                ChannelPair::InPlace(b) => Some(b),
                ChannelPair::InputOutput(i, o) => {
                    o.copy_from_slice(i);
                    Some(o)
                }
            }
        }

        #[allow(unused_variables)]
        for event_batch in events.input.batch() {

            for event in event_batch.events() {
                self.handle_event(event);
            }

            // 音量を0.5倍にする
            for buf in channel_buffers.iter_mut().flatten() {
               for sample in buf.iter_mut() {
                   *sample *= 0.5;
               }
            }
        }

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }
}

impl ClapTestAudioProcessor<'_> {
    fn handle_event(&mut self, event: &UnknownEvent) {
        match event.as_core_event() {
            Some(_core_event) => {
                todo!()
            }
            None => {
                todo!()
            }
        }
    }
}

pub struct ClapTestShared {}

impl<'a> PluginShared<'a> for ClapTestShared {}

pub struct ClapTestMainThread<'a> {
    #[allow(dead_code)]
    shared: &'a ClapTestShared,
}

impl<'a> PluginAudioPortsImpl for ClapTestMainThread<'a> {
    fn count(&mut self, _is_input: bool) -> u32 {
        1
    }

    fn get(&mut self, index: u32, _is_input: bool, writer: &mut AudioPortInfoWriter) {
        if index == 0 {
            writer.set(&AudioPortInfo {
                id: ClapId::new(0),
                name: b"main",
                channel_count: 2,
                flags: AudioPortFlags::IS_MAIN,
                port_type: Some(AudioPortType::STEREO),
                in_place_pair: None,
            })
        }
    }
}

impl<'a> PluginNotePortsImpl for ClapTestMainThread<'a> {
    fn count(&mut self, is_input: bool) -> u32 {
        if is_input {
            1
        } else {
            0
        }
    }

    fn get(&mut self, index: u32, is_input: bool, writer: &mut NotePortInfoWriter) {
        if is_input && index == 0 {
            writer.set(&NotePortInfo {
                id: ClapId::new(1),
                name: b"main",
                preferred_dialect: Some(NoteDialect::Clap),
                supported_dialects: NoteDialects::CLAP,
            })
        }
    }
}

impl<'a> PluginMainThread<'a, ClapTestShared> for ClapTestMainThread<'a> {}

clack_export_entry!(SinglePluginEntry<ClapTest>);