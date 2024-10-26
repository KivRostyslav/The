use std::io::{stdin, stdout};

use crossbeam::channel::Sender;
use termion::{input::TermRead, raw::IntoRawMode};
use crate::controller::Controller;

#[derive(Debug, Clone, Copy)]
pub enum LoopEvent {
    PlayPause,
    Skip(i32),
    Shutdown,
}

pub struct EventLoopController<'a> {
    event_loop_senders: &'a[Sender<LoopEvent>],
}

impl<'a> EventLoopController<'a> {
    pub fn new(event_loop_senders: &'a[Sender<LoopEvent>]) -> Self {
        Self { event_loop_senders }
    } 

    fn send(&self, event: LoopEvent) {
        for x in self.event_loop_senders {
            x.send(event).unwrap();
        }
    }

}

impl<'a> Controller for EventLoopController<'a> {
    fn run(&mut self) {
        let _stdout = stdout().into_raw_mode().unwrap();
        let mut keys = stdin().keys();

        loop {
            let key = keys.next().unwrap();
            let event = match key.unwrap() {
                termion::event::Key::Char(' ') | termion::event::Key::Char('k') => { LoopEvent::PlayPause },
                termion::event::Key::Char('j') => { LoopEvent::Skip(-10) },
                termion::event::Key::Char('l') => { LoopEvent::Skip(10) },
                termion::event::Key::Ctrl('c') => {
                    self.send(LoopEvent::Shutdown);
                    break;
                }
                _ => { continue; },
            };

            self.send(event);
        }
    }
}
