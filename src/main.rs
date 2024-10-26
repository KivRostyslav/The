mod args;
mod event_loop;
mod media;
mod audio;
mod terminal;
mod ascii;
mod controller;

use std::env;

use audio::AudioController;
use crossbeam::channel::unbounded;
use event_loop::{EventLoopController, LoopEvent};
use media::MediaController;
use opencv::prelude::*;
use terminal::{StringInfo, TerminalController};
use crate::controller::Controller;

fn main() {
    println!("Hello, world!");
    let (tx_frame, rx_frame) = unbounded::<StringInfo>();
    let (txs_event, rxs_event): (Vec<_>, Vec<_>) = (0..3).map(|_| unbounded::<LoopEvent>()).unzip();

    let args: Vec<String> = env::args().collect();

    //let mut audio_controller = AudioController::new(&args[1], &rxs_event[0]);
    let mut media_controller = MediaController::new(&String::from(r#"https://muxer.sf-converter.com/get/QlprOFVaN2E3dVl8VHdvIFRpbWUgfCBBbmltYXRpb24gTWVtZXwxNzI5NjA1NDYy.b6234ac485634fab3b534a2f6a10ace4"#), &tx_frame, &rxs_event[1]).unwrap();
    let mut terminal_controller = TerminalController::new(&rx_frame, &rxs_event[2]);
    let mut event_loop_controller = EventLoopController::new(&txs_event);

    let _x = crossbeam::scope(|x| {
        x.spawn(move |_| {
            media_controller.run();
        });
        x.spawn(move |_| {
            terminal_controller.run();
        });
        x.spawn(move |_| {
            //audio_controller.run();
        });
        x.spawn(move |_| {
            event_loop_controller.run();
        });
    });
}



