use iced::keyboard;
use iced::keyboard::{Event, KeyCode};

#[derive(Debug, Copy, Clone)]
pub enum Message {
    /// true -> forwards, false -> backwards
    NextField(bool),
}

pub fn handle(event: keyboard::Event) -> Option<crate::Message> {
    type Modifiers = (bool, bool, bool);
    // const CTRL: Modifiers = (true, false, false);
    const SHIFT: Modifiers = (false, false, true);
    // const CTRL_ALT: Modifiers = (true, true, false);
    // const CTRL_SHIFT: Modifiers = (true, false, true);
    const NONE: Modifiers = (false, false, false);

    match event {
        keyboard::Event::KeyPressed { key_code, modifiers } => {
            let modifiers = (modifiers.control, modifiers.alt, modifiers.shift);
            // let message = match (modifiers.control, modifiers.alt, modifiers.shift) {
            //     _ => None,
            // };
            let message = match key_code {
                KeyCode::Tab => match modifiers {
                    NONE => Some(Message::NextField(true)),
                    SHIFT => Some(Message::NextField(false)),
                    _ => None,
                }
                _ => None,
            };
            message.map(crate::Message::HotKey)
        }
        Event::KeyReleased { .. } => None,
        _ => None,
    }
}