use std::{io::{stdout, Stdout, Write}, thread::sleep, time::Duration};

use crossbeam::channel::{Receiver, Sender};
use termion;

#[derive(Debug)]
pub struct StringInfo {
    pub string: Vec<u8>,
    pub rgb: Vec<u8>,
}

pub struct TerminalController<'a> {
    media_receiver: &'a Receiver<StringInfo>,
    terminal_sender: &'a Sender<TerminalEvents>,
}

pub enum TerminalEvents {
    Skip(i32),
    Play(bool),
}

impl<'a> TerminalController<'a> { 
    pub fn new(media_receiver: &'a Receiver<StringInfo>, terminal_sender: &'a Sender<TerminalEvents>) -> Self {
        Self {
            media_receiver,
            terminal_sender,
        }
    }
    
    pub fn run(&self) {
        //print!("{}", termion::cursor::Hide);
        let mut stdout = stdout();
        loop {
            let string = self.media_receiver.recv().unwrap();
            let _ = stdout.lock();

            if string.rgb.is_empty() {
                write!(&stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
                stdout.write_all(&string.string).unwrap();
                stdout.flush().unwrap();
                continue;
            }
            /*
            let mut out = String::new(); 
            let mut current_color: (u8, u8, u8) = (0, 0, 0); 
            let mut out = String::new();
            let mut rgb_index = 0;
            for char in string.string.chars() {
                if char == '\n' {
                    continue;
                }                    

                if current_color != (string.rgb[rgb_index], string.rgb[rgb_index + 1], string.rgb[rgb_index + 2]) {
                    out.push_str(format!("\x1B[38;2;{};{};{}m", string.rgb[rgb_index + 2], string.rgb[rgb_index + 1], string.rgb[rgb_index]).as_str());     
                    current_color = (string.rgb[rgb_index], string.rgb[rgb_index + 1], string.rgb[rgb_index + 2]);
                } 

                out.push(char);    
                rgb_index += 3;
            }
            
            print!("{}{}{}", termion::cursor::Goto(1, 1), out, termion::cursor::Goto(1, 1));*/
        }
    }
}
