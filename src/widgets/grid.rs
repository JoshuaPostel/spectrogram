use iced::{
    canvas::{self, Cache, Canvas, Cursor, Frame, Geometry, Path},
    Color, Element, Length, Point, Rectangle, Size,
};

use fftw::array::AlignedVec;
use fftw::plan::{R2CPlan, R2CPlan64};
use fftw::types::Flag;

use crate::messages::Message;
use crate::units::{format_unit, map_normalized, normalize, Mapping, Scale};

pub struct Grid {
    resolution: (u32, u32),
    x: Scale,
    pub y: Scale,
    fill_proportion: u16,
    frequencies: Vec<f64>,
    pub cache: Cache,
}

impl Grid {
    pub fn new(width: u32, height: u32, fill_proportion: u16, x: Scale, y: Scale) -> Grid {
        let frequencies: Vec<f64> = vec![];
        Grid {
            resolution: (width, height),
            x,
            y,
            fill_proportion,
            frequencies,
            cache: Cache::new(),
        }
    }

    pub fn update_frequencies(&mut self, resolution: (u32, u32), samples: &Vec<i16>) {
        self.resolution = resolution;
        self.calculate_frequencies(samples);
        self.cache.clear();
    }

    pub fn view<'a>(&'a mut self) -> Element<'a, Message> {
        let fill_proportion = self.fill_proportion;
        Canvas::new(self)
            .width(Length::FillPortion(fill_proportion))
            .height(Length::FillPortion(fill_proportion))
            .into()
    }

    fn calculate_frequencies(&mut self, samples: &Vec<i16>) {
        self.frequencies = vec![];
        let n_columns = self.resolution.0 as usize;
        let n_rows = self.resolution.1 as usize * 2;

        let f64_samples: Vec<f64> = samples.iter().map(|x| *x as f64).collect();

        let mut plan: R2CPlan64 =
            R2CPlan::aligned(&[n_rows], Flag::MEASURE).expect("plan to create");
        let mut inputs = AlignedVec::new(n_rows);
        let mut outputs = AlignedVec::new(n_rows / 2 + 1);

        for column in 0..n_columns {
            let start = column * n_rows;
            let end = (column + 1) * n_rows;
            inputs.copy_from_slice(&f64_samples[start..end]);
            plan.r2c(&mut inputs, &mut outputs)
                .expect("fftw dft to execute");
            let real: Vec<f64> = outputs.iter().map(|x| x.norm()).collect();
            let max = real.iter().map(|x| *x as u64).max().unwrap() as f64;
            let mut normalized: Vec<f64> = real.iter().map(|x| x / max).collect();
            self.frequencies.append(&mut normalized);
        }
    }
}

impl canvas::Program<Message> for Grid {
    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let grid = self.cache.draw(bounds.size(), |frame| {
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::from_rgb(0.0, 0.0, 0.0));

            let n_rows = self.resolution.0;
            let n_columns = self.resolution.1 + 1;

            let linear_y_scale = Scale {
                unit: self.y.unit,
                min: self.y.min,
                max: self.y.max,
                mapping: Mapping::Linear,
            };
            let mut y_positions: Vec<f32> = linear_y_scale
                .evenly_spaced_values(n_columns as usize, false)
                .iter()
                .map(|value| normalize(*value, &self.y))
                .map(|normalized| normalized * bounds.height)
                .collect();
            y_positions.push(bounds.height);

            let linear_x_scale = Scale {
                unit: self.x.unit,
                min: self.x.min,
                max: self.x.max,
                mapping: Mapping::Linear,
            };
            let mut x_positions: Vec<f32> = linear_x_scale
                .evenly_spaced_values(n_rows as usize, false)
                .iter()
                .map(|value| normalize(*value, &self.x))
                .map(|normalized| normalized * bounds.width)
                .collect();
            x_positions.push(bounds.width);

            let mut index = 0;
            for row in 0..n_rows {
                for column in 0..n_columns {
                    let height = y_positions[(column + 1) as usize] - y_positions[column as usize];
                    let cell = Size::new(bounds.width / (n_rows as f32), -height);
                    let point = Point::new(
                        x_positions[row as usize],
                        bounds.height - y_positions[column as usize],
                    );
                    let inner_rec = Path::rectangle(point, cell);
                    let intensity = self.frequencies[index] as f32;
                    frame.fill(&inner_rec, Color::from_rgb(intensity, 0.0, intensity));
                    index += 1;
                }
            }
        });

        let cursor_position = cursor.position().unwrap_or(Point::new(0.0, 0.0));

        let normalized_x = (cursor_position.x - bounds.x) / bounds.width;
        let normalized_y = 1.0 - (cursor_position.y - bounds.y) / bounds.height;

        let x_unit = format_unit(map_normalized(normalized_x, &self.x), &self.x.unit);
        let y_unit = format_unit(map_normalized(normalized_y, &self.y), &self.y.unit);

        if bounds.contains(cursor_position) {
            let overlay = {
                let mut frame = Frame::new(bounds.size());
                let text_position = Point::new(0.0, bounds.height);
                let text = canvas::Text {
                    position: text_position,
                    content: format!("x: {}\ny: {}", x_unit, y_unit),
                    color: Color::WHITE,
                    vertical_alignment: iced::VerticalAlignment::Bottom,
                    horizontal_alignment: iced::HorizontalAlignment::Left,
                    ..Default::default()
                };
                frame.fill_text(text);
                frame.into_geometry()
            };

            vec![grid, overlay]
        } else {
            vec![grid]
        }
    }
}
