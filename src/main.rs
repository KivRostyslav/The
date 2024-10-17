mod args;
mod media;
mod audio;
mod terminal;
mod ascii;

use std::env;

use audio::AudioController;
use crossbeam::channel::unbounded;
use media::MediaController;
use opencv::prelude::*;
use terminal::TerminalController;

fn main() {
    println!("Hello, world!");
    let (media_sender, media_receiver) = unbounded();
    let (terminal_sender, terminal_receiver) = unbounded();

    let args: Vec<String> = env::args().collect();

    let audio_controller = AudioController::new(&args[1]);
    let media_controller = MediaController::new(&args[1], &media_sender, &terminal_receiver);
    let terminal_controller = TerminalController::new(&media_receiver, &terminal_sender);

    let x = crossbeam::scope(|x| {
        x.spawn(move |_| {
            media_controller.unwrap().run();
        });
        x.spawn(move |_| {
            terminal_controller.run();
        });
        x.spawn(move |_| {
            audio_controller.run();
        });
    });
}



