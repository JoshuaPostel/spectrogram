use iced::mouse::Event::CursorMoved;
use iced::Point;

use iced_native::event::Event;

use super::units::{Mapping, Unit};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    SliderChanged(u32),
    FileButtonPressed,
    CursorMoved(Point),
    YUnitChanged(Unit),
    YMappingChanged(Mapping),
    ActiveChannelChanged(usize),
    DynamicAxesChanged(bool),
}

pub fn cursor_moved_filter(event: Event, _: iced_native::event::Status) -> Option<Message> {
    match event {
        Event::Mouse(e) => match e {
            CursorMoved { position } => Some(Message::CursorMoved(position)),
            _ => None,
        },
        _ => None,
    }
}
