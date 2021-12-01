use iced::{Color, Element, Point, Rectangle, Size};
use iced_audio::core::offset::Offset;
use iced_audio::graphics::text_marks;
use iced_audio::graphics::tick_marks;
use iced_audio::native::tick_marks::Tier;
use iced_audio::style;
use iced_graphics::Primitive;
use iced_native::Length;

use iced::canvas;
use iced::canvas::{Cache, Canvas, Cursor, Geometry};

use crate::messages::Message;
use crate::units::{format_unit, Scale};

pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct Axis {
    orientation: Orientation,
    pub scale: Scale,
    pub tick_count: usize,
    fill_proportion: u16,
    pub cache: Cache,
}

impl Axis {
    pub fn new(
        orientation: Orientation,
        scale: Scale,
        tick_count: usize,
        fill_proportion: u16,
    ) -> Self {
        Axis {
            orientation,
            scale,
            tick_count,
            fill_proportion,
            cache: Cache::new(),
        }
    }

    pub fn view<'a>(&'a mut self) -> Element<'a, Message> {
        let fill_proportion = self.fill_proportion;
        Canvas::new(self)
            .width(Length::FillPortion(fill_proportion))
            .height(Length::FillPortion(fill_proportion))
            .into()
    }
}

impl canvas::Program<Message> for Axis {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let axis = self.cache.draw(bounds.size(), |frame| {
            let width = bounds.size().width;
            let height = bounds.size().height;

            //let label_values = &self.scale.evenly_spaced_values(self.tick_count, true);
            let label_values = &self.scale.evenly_spaced_values(16, true);
            let labels: Vec<String> = label_values
                .iter()
                .map(|f| format_unit(*f, &self.scale.unit))
                .collect();

            // I dont believe there is a way around this extra allocation
            let str_labels: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

            let text_marks = text_marks::Group::evenly_spaced(&str_labels[..]);

            let ticks = tick_marks::Group::evenly_spaced(self.tick_count, Tier::One);
            let axis_line: Rectangle;
            let rendered_tick_marks: Primitive;
            let rendered_text_marks: Primitive;
            match self.orientation {
                Orientation::Horizontal => {
                    let tick_bounds =
                        Rectangle::new(Point::new(0.0, 0.0), Size::new(width, height / 2.0));
                    let text_bounds = Rectangle::new(
                        Point::new(0.0, height / 2.0),
                        Size::new(width, height / 2.0),
                    );
                    axis_line =
                        Rectangle::new(Point::new(0.0, height / 4.0), Size::new(width, 1.0));
                    rendered_tick_marks = tick_marks::draw_horizontal_tick_marks(
                        &tick_bounds,
                        &ticks,
                        &THIN_TICKS,
                        &style::tick_marks::Placement::Center {
                            offset: Offset::ZERO,
                            fill_length: true,
                        },
                        false,
                        &tick_marks::PrimitiveCache::default(),
                    );
                    rendered_text_marks = text_marks::draw_horizontal_text_marks(
                        &text_bounds,
                        &text_marks,
                        &style::text_marks::Style::default(),
                        &style::text_marks::Placement::Center {
                            align: style::text_marks::Align::Center,
                            offset: Offset::ZERO,
                        },
                        false,
                        &text_marks::PrimitiveCache::default(),
                    );
                }
                Orientation::Vertical => {
                    let tick_bounds = Rectangle::new(
                        Point::new(width / 2.0, 1.0),
                        Size::new(width / 2.0, height - 1.0),
                    );
                    let text_bounds =
                        Rectangle::new(Point::new(0.0, 0.0), Size::new(width / 2.0, height));
                    axis_line =
                        Rectangle::new(Point::new(width * 0.75, 0.0), Size::new(1.0, height));
                    rendered_tick_marks = tick_marks::draw_vertical_tick_marks(
                        &tick_bounds,
                        &ticks,
                        &THIN_TICKS,
                        &style::tick_marks::Placement::Center {
                            offset: Offset::ZERO,
                            fill_length: true,
                        },
                        false,
                        &tick_marks::PrimitiveCache::default(),
                    );
                    rendered_text_marks = text_marks::draw_vertical_text_marks(
                        &text_bounds,
                        &text_marks,
                        &style::text_marks::Style::default(),
                        &style::text_marks::Placement::Center {
                            align: style::text_marks::Align::Center,
                            offset: Offset::ZERO,
                        },
                        false,
                        &text_marks::PrimitiveCache::default(),
                    );
                }
            }
            frame.fill_rectangle(axis_line.position(), axis_line.size(), Color::BLACK);
            fill_from_primitive(rendered_tick_marks, frame);
            fill_from_primitive(rendered_text_marks, frame);
        });
        vec![axis]
    }
}

// renderes primities created iced_audio by onto a frame
// TODO consider removing iced_audio dependancy or developing a cleaner solution
fn fill_from_primitive(primitive: Primitive, frame: &mut canvas::Frame) {
    match primitive {
        Primitive::Group { primitives } => {
            for primitive in primitives {
                match primitive {
                    Primitive::Quad { bounds, .. } => {
                        frame.fill_rectangle(bounds.position(), bounds.size(), Color::BLACK);
                    }
                    Primitive::Text {
                        content,
                        bounds,
                        color,
                        size,
                        font,
                        horizontal_alignment,
                        vertical_alignment,
                    } => {
                        let text = canvas::Text {
                            content,
                            position: bounds.position(),
                            color,
                            size,
                            font,
                            horizontal_alignment,
                            vertical_alignment,
                        };
                        frame.fill_text(text);
                    }
                    _ => (), // did not find a quad or text
                }
            }
        }
        Primitive::Cached { cache } => {
            match &*cache {
                Primitive::Group { primitives } => {
                    for primitive in primitives {
                        match primitive {
                            Primitive::Quad { bounds, .. } => {
                                frame.fill_rectangle(
                                    bounds.position(),
                                    bounds.size(),
                                    Color::BLACK,
                                );
                            }
                            Primitive::Text {
                                content,
                                bounds,
                                color,
                                size,
                                font,
                                horizontal_alignment,
                                vertical_alignment,
                            } => {
                                let text = canvas::Text {
                                    content: content.to_string(),
                                    position: bounds.position(),
                                    color: *color,
                                    size: *size,
                                    font: *font,
                                    horizontal_alignment: *horizontal_alignment,
                                    vertical_alignment: *vertical_alignment,
                                };
                                frame.fill_text(text);
                            }
                            _ => (), // did not find a quad or text
                        }
                    }
                }
                _ => (), // did not find a group in cache
            }
        }
        _ => (), // did not find a group
    }
}

const THIN_TICKS: style::tick_marks::Style = style::tick_marks::Style {
    tier_1: style::tick_marks::Shape::Line {
                length: 4.0,
                width: 1.0,
                color: Color::BLACK,
            },
            tier_2: style::tick_marks::Shape::Line {
                length: 3.0,
                width: 1.0,
                color: Color::BLACK,
            },
            tier_3: style::tick_marks::Shape::Line {
                length: 2.0,
                width: 1.0,
                color: Color::BLACK,
            },
};
