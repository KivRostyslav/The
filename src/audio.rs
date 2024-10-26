use std::fs::File;
use std::io::BufReader;
use crossbeam::channel::Receiver;
use rodio::{Sink, Decoder, OutputStream};
use crate::controller::Controller;

use crate::event_loop::LoopEvent;

pub struct AudioController<'a> {
    sink: Sink,
    event_loop_receiver: &'a Receiver<LoopEvent>,
}

impl<'a> AudioController<'a> {
    pub fn new(path: &String, event_loop_receiver: &'a Receiver<LoopEvent>) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let file = BufReader::new(File::open(path.clone()).unwrap());
        let source = Decoder::new(file).unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(source);

        Self {
            sink, 
            event_loop_receiver,
        }
    }
}

impl<'a> Controller for AudioController<'a> {
    fn run(&mut self) {
        self.sink.play();
        loop {
            let event = self.event_loop_receiver.recv().unwrap();
            println!("audio {:?}", event);
            match event {
                LoopEvent::PlayPause => {
                    if self.sink.is_paused() {
                        self.sink.play();
                        continue;
                    }

                    self.sink.pause();
                },
                LoopEvent::Shutdown => {
                    self.sink.stop();
                    break;
                },
                _ => { },
            }
        }
    }
}
