use std::borrow::Cow;

use iced::{
    button, executor, pick_list, slider, Align, Application, Button, Clipboard, Column, Command,
    Container, Element, Length, PickList, Radio, Row, Settings, Slider, Text,
};

use iced_native::subscription::Subscription;
use rfd::{FileDialog, MessageButtons, MessageDialog};

use spectrogram::io::wav::WAV;
use spectrogram::messages::{cursor_moved_filter, Message};
use spectrogram::units::{Mapping, Scale, Unit};
use spectrogram::widgets::axis::{Axis, Orientation};
use spectrogram::widgets::grid::Grid;

fn main() -> iced::Result {
    Spectrogram::run(Settings::default())
}

struct Spectrogram {
    wav: WAV,
    n_samples: usize,
    samples: Vec<i16>,
    // TODO resolution to u32?
    resolution: (u32, u32),
    slider: slider::State,
    grid: Grid,
    x_axis: Axis,
    y_axis: Axis,
    dynamic_axes: bool,
    file_button: button::State,
    active_channel: usize,
    active_channel_pick_list: pick_list::State<usize>,
}

impl Spectrogram {
    fn new(wav: WAV, number_of_samples: Option<usize>, width: u32) -> Spectrogram {
        let n_samples: usize;
        match number_of_samples {
            Some(n) => n_samples = n,
            None => {
                n_samples = wav.data_header.size as usize / (wav.fmt_header.nchannels as usize * 2)
            }
        }
        let sample_rate = wav.fmt_header.sample_rate;
        let max_time = (1.0 / sample_rate as f32) * n_samples as f32;
        let max_frequency = (sample_rate / 2) as f32;

        let active_channel = 0;
        let samples = wav.channels[active_channel]
            .iter()
            .map(|x| *x as i16)
            .collect::<Vec<i16>>();

        let height = ((n_samples as u32) / width) / 2;
        let x_scale = Scale {
            min: 0.0,
            max: max_time,
            unit: Unit::Second,
            mapping: Mapping::Linear,
        };
        let y_scale = Scale {
            min: 0.0,
            max: max_frequency,
            unit: Unit::Note,
            mapping: Mapping::Log10,
        };

        let mut spectrogram = Spectrogram {
            wav,
            n_samples,
            samples,
            resolution: (width, height),
            slider: slider::State::new(),
            grid: Grid::new(width, height, 20, x_scale.clone(), y_scale.clone()),
            x_axis: Axis::new(Orientation::Horizontal, x_scale, 16, 20),
            y_axis: Axis::new(Orientation::Vertical, y_scale, 16, 1),
            dynamic_axes: false,
            file_button: button::State::new(),
            active_channel,
            active_channel_pick_list: pick_list::State::default(),
        };
        spectrogram
            .grid
            .update_frequencies(spectrogram.resolution, &spectrogram.samples);
        spectrogram
    }

    fn update_wav(&mut self, wav: WAV) {
        let sample_rate = wav.fmt_header.sample_rate;

        self.n_samples = wav.data_header.size as usize / (wav.fmt_header.nchannels as usize * 2);
        self.wav = wav;
        self.x_axis.scale.max = (1.0 / sample_rate as f32) * self.n_samples as f32;
        self.y_axis.scale.max = (sample_rate / 2) as f32;
        self.resolution.1 = ((self.n_samples as u32) / self.resolution.0) / 2;

        self.active_channel = 0;
        self.samples = self.wav.channels[0]
            .iter()
            .map(|x| *x as i16)
            .collect::<Vec<i16>>();
        self.grid.update_frequencies(self.resolution, &self.samples);
        self.x_axis.cache.clear();
        self.y_axis.cache.clear();
    }

    fn update_channel(&mut self, channel: usize) {
        self.active_channel = channel;
        self.samples = self.wav.channels[channel]
            .iter()
            .map(|x| *x as i16)
            .collect::<Vec<i16>>();
        self.grid.update_frequencies(self.resolution, &self.samples);
        self.grid.cache.clear();
    }

    fn update_resolution(&mut self, width: u32) {
        let height = ((self.n_samples as u32) / width) / 2;
        self.resolution = (width, height)
    }
}

impl Application for Spectrogram {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let bytes = std::include_bytes!("demo.wav");
        let wav = WAV::from(&bytes[..]).unwrap();
        (Spectrogram::new(wav, None, 100), Command::none())
    }

    fn title(&self) -> String {
        String::from("spectrogram")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        match message {
            Message::CursorMoved(_point) => (),
            Message::SliderChanged(value) => {
                self.update_resolution(value);
                self.grid.update_frequencies(self.resolution, &self.samples);
                if self.dynamic_axes {
                    self.x_axis.tick_count = self.resolution.0 as usize + 1;
                    self.y_axis.tick_count = self.resolution.1 as usize + 2;
                }
                self.x_axis.cache.clear();
                self.y_axis.cache.clear();
            }
            Message::FileButtonPressed => {
                let file = FileDialog::new()
                    .add_filter("WAV", &["wav", "WAV"])
                    .pick_file();

                match file {
                    Some(file) => {
                        let filename = file.to_str().expect("good filename");
                        let wav = WAV::from_file(filename);
                        match wav {
                            Ok(wav) => self.update_wav(wav),
                            Err(e) => {
                                MessageDialog::new()
                                    .set_title(&format!("Error loading: {}", filename))
                                    .set_description(&format!("Application error message:\n{}", e))
                                    .set_buttons(MessageButtons::OkCancel)
                                    .show();
                            }
                        }
                    }
                    None => (),
                }
            }
            Message::YUnitChanged(unit) => {
                self.grid.y.unit = unit;
                self.y_axis.scale.unit = unit;
                self.y_axis.cache.clear();
            }
            Message::YMappingChanged(mapping) => {
                self.y_axis.scale.mapping = mapping;
                self.y_axis.cache.clear();
                self.grid.y.mapping = mapping;
                self.grid.cache.clear();
            }
            Message::ActiveChannelChanged(channel) => {
                self.update_channel(channel);
            }
            Message::DynamicAxesChanged(dynamic_axes) => {
                self.dynamic_axes = dynamic_axes;
                if self.dynamic_axes {
                    self.x_axis.tick_count = self.resolution.0 as usize + 1;
                    self.y_axis.tick_count = self.resolution.1 as usize + 2;
                } else {
                    self.x_axis.tick_count = 16;
                    self.y_axis.tick_count = 16;
                }
                self.y_axis.cache.clear();
                self.x_axis.cache.clear();
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events_with(cursor_moved_filter)
    }

    fn view(&mut self) -> Element<Message> {
        let slider = Slider::new(
            &mut self.slider,
            1..=100,
            self.resolution.0,
            Message::SliderChanged,
        );

        let y_unit = self.y_axis.scale.unit.clone();
        let y_mapping = self.y_axis.scale.mapping.clone();

        let row1 = Row::new()
            .height(Length::FillPortion(20))
            .push(self.y_axis.view())
            .push(self.grid.view());

        let spacer = Row::new().width(Length::FillPortion(1));

        let row2 = Row::new()
            .height(Length::FillPortion(2))
            .push(spacer)
            .push(self.x_axis.view());

        let dynamic_axes_controls = Column::new()
            .spacing(1)
            .push(Text::new("Axes"))
            .push(
                Radio::new(
                    false,
                    "Static",
                    Some(self.dynamic_axes),
                    Message::DynamicAxesChanged,
                )
                .size(20)
                .spacing(5),
            )
            .push(
                Radio::new(
                    true,
                    "Resolution",
                    Some(self.dynamic_axes),
                    Message::DynamicAxesChanged,
                )
                .size(20)
                .spacing(5),
            );

        let choices: Vec<usize> = (0..(self.wav.fmt_header.nchannels as usize)).collect();
        let active_channel_pick_list = PickList::new(
            &mut self.active_channel_pick_list,
            Cow::Owned(choices),
            Some(self.active_channel),
            Message::ActiveChannelChanged,
        );

        let y_unit_controls = Column::new()
            .spacing(1)
            .push(Text::new("Unit"))
            .push(
                Radio::new(Unit::Note, "Note", Some(y_unit), Message::YUnitChanged)
                    .size(20)
                    .spacing(5),
            )
            .push(
                Radio::new(Unit::Hz, "Hz", Some(y_unit), Message::YUnitChanged)
                    .size(20)
                    .spacing(5),
            );

        let y_mapping_controls = Column::new()
            .spacing(1)
            .push(Text::new("Scale"))
            .push(
                Radio::new(
                    Mapping::Linear,
                    "Linear",
                    Some(y_mapping),
                    Message::YMappingChanged,
                )
                .size(20)
                .spacing(5),
            )
            .push(
                Radio::new(
                    Mapping::Log10,
                    "Log10",
                    Some(y_mapping),
                    Message::YMappingChanged,
                )
                .size(20)
                .spacing(5),
            );

        let y_resolution_controls = Column::new()
            .spacing(1)
            .push(Text::new("Resolution"))
            .push(slider);

        let controls = Row::new()
            .height(Length::FillPortion(2))
            .align_items(Align::Center)
            .spacing(20)
            .push(
                Button::new(&mut self.file_button, Text::new("Load .wav file"))
                    .on_press(Message::FileButtonPressed),
            )
            .push(Text::new("Channel:"))
            .push(active_channel_pick_list)
            .push(dynamic_axes_controls)
            .push(Text::new("Y-axis:"))
            .push(y_unit_controls)
            .push(y_mapping_controls)
            .push(y_resolution_controls);

        let column = Column::new().push(row1).push(row2).push(controls);

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }
}
