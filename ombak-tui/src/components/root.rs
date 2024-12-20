use std::sync::mpsc::Sender;

use crate::component::Component;
use crate::render::Message;
use crate::widgets::Waveform;

use crossterm::event::{KeyCode, KeyEvent};

use ratatui::layout::Rect;
use ratatui::Frame;

pub struct Root {
    message_tx: Sender<Message>,
    width: u8,
    height: u16,
}

impl Root {
    pub fn new(message_tx: Sender<Message>) -> Self {
        Self {
            message_tx,
            width: 3,
            height: 1,
        }
    }

    fn notify_render(&self) {
        self.message_tx.send(Message::Render).unwrap();
    }

    fn notify_quit(&self) {
        self.message_tx.send(Message::Quit).unwrap();
    }
}

impl Component for Root {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        f.render_widget(
            Waveform::new(
                vec!["fuad ismail".to_string(), "sri umay".to_string()],
                self.height,
                self.width,
            ),
            rect,
        );
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Up => self.height = u16::saturating_add(self.height, 1),
            KeyCode::Down => self.height = u16::saturating_sub(self.height, 1),
            KeyCode::Right => self.width = u8::saturating_add(self.width, 1),
            KeyCode::Left => self.width = u8::saturating_sub(self.width, 1),
            KeyCode::Char('q') => {
                self.notify_quit();
                return;
            }
            _ => (),
        }
        self.notify_render();
    }
}