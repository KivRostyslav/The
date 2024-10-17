use core::panic;
use std::fs::File;
use std::io::BufReader;
use rodio::{Sink, Decoder, OutputStream, source::Source};

pub struct AudioController {
    path: String,
}

impl AudioController {
    pub fn new(path: &String) -> Self {
        
        Self {
            path: path.clone(),
        }
    }

    pub fn run(&self) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let file = BufReader::new(File::open(self.path.clone()).unwrap());
        let source = Decoder::new(file).unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(source);
        sink.sleep_until_end();
        panic!("jsdfajdlfsflsdak");
    }
}
