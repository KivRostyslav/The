use std::io::{stdout, Write};

use crossbeam::channel::Receiver;
use crate::event_loop::LoopEvent;
use crate::controller::Controller;

pub struct StringInfo {
    pub char_len: u32,
    pub string: Vec<u8>,
    pub rgb: Vec<u8>,
}

pub struct TerminalController<'a> {
    media_receiver: &'a Receiver<StringInfo>,
    event_loop_receiver: &'a Receiver<LoopEvent>,
}

impl<'a> TerminalController<'a> { 
    pub fn new(media_receiver: &'a Receiver<StringInfo>, event_loop_receiver: &'a Receiver<LoopEvent>) -> Self {
        Self { 
            media_receiver,
            event_loop_receiver,
        }
    }
}

impl<'a> Controller for TerminalController<'a> {
    fn run(&mut self) {
        print!("{}", termion::cursor::Hide);
        let mut stdout = stdout();
        loop {
            if !self.event_loop_receiver.is_empty() {
                let event = self.event_loop_receiver.recv().unwrap();
                if let LoopEvent::Shutdown = event {
                    break;
                }
            }

            let string = self.media_receiver.recv().unwrap();
            let mut locked = stdout.lock();

            write!(&stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
            if string.rgb.is_empty() {
                locked.write_all(&string.string).unwrap();
                locked.flush().unwrap();
                continue;
            }
            
            let mut current_color: (u8, u8, u8) = (0, 0, 0); 
            let mut last_color_change_index = 0;
            let mut rgb_index = 0;
            for index in (0..string.string.len()).step_by(string.char_len as usize) {
                if current_color != (string.rgb[rgb_index], string.rgb[rgb_index + 1], string.rgb[rgb_index + 2]) {
                    locked.write_all(&string.string[last_color_change_index..index]).unwrap();
                    last_color_change_index = index;

                    write!(&mut locked, "{}", format!("\x1B[38;2;{};{};{}m", string.rgb[rgb_index + 2], string.rgb[rgb_index + 1], string.rgb[rgb_index]).as_str()).unwrap();     
                    current_color = (string.rgb[rgb_index], string.rgb[rgb_index + 1], string.rgb[rgb_index + 2]);
                } 

                rgb_index += 3;
            }
            write!(&mut locked, "{}", termion::cursor::Goto(1, 1));
            stdout.flush().unwrap();
        }

        print!("{}", termion::cursor::Show);
    }
}
