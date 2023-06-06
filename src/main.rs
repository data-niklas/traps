use clap::{App, Arg};
use lazy_static::*;
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::fs::File;
use std::process::Command;

mod constants;
mod fifo;
mod lib;
mod ui;
mod config;


const FIFO_PATH: &str = "/tmp/traps0.1.2";

fn main() {
    let matches = make_app().get_matches();
    if matches.is_present("command") {
        write_to_fifo(matches.value_of("command").expect("Should have a value"));
        std::process::exit(0);
    }
    init();
}

fn init() {
    let fifo = fifo::Fifo::new(PathBuf::from(FIFO_PATH));
    if !fifo.exists(){
        fifo.create();
    }

    let mut window;
    let mut recorder;
    {

        let config = config::Config::new();
    
        window = ui::UI::new(&config);
        recorder = lib::GestureRecorder::new(Box::new(move|gesture|{
            let value = &gesture.action;
            Command::new ("/bin/sh").args (&["-c", value]).spawn();
            true
        }));
        //let mut recorder = recorder_ref.lock().expect("Recorder should not be locked");
        for gesture in config.gestures{
            recorder.register_gesture(gesture);
        }
    }


    window.init();
    loop {
        if read_from_fifo(&mut fifo.open_read()){
            break;
        }
    }
    window.set_visible(true);

    //let recorder_closure = recorder;
    window.event_loop(Box::new(move |event| {
        //let mut recorder = recorder_closure;
        match event{
            ui::Event::Point(x, y) => {
                if recorder.is_tracking{
                    recorder.track(lib::Point::new(x, y));
                }
            }
            ui::Event::Start => {
                recorder.start();
            }
            ui::Event::Stop => {
                recorder.stop();
                loop {
                    if read_from_fifo(&mut fifo.open_read()){
                        break;
                    }
                }
            }
        }
    }));
}

fn read_from_fifo(fifo_file: &mut File) -> bool{
    let mut buf = String::new();
    fifo_file.read_to_string(&mut buf);
    for line in buf.lines(){
        match line{
            "show" => {
                return true;
            }
            "stop" => {
                std::process::exit(0);
            }
            _ => {}
        }
    }
    false
}

fn write_to_fifo(text: &str) {
    let pipe = fifo::Fifo::new(PathBuf::from(FIFO_PATH));
    if pipe.exists() {
        let mut file = pipe.open_write();
        file.write_all(text.as_bytes()).expect("Could not write to fifo ");
    } else {
        std::process::exit(1);
    }
}

fn make_app() -> clap::App<'static> {
    App::new(constants::APPNAME)
        .version(constants::VERSION)
        .author(constants::AUTHOR)
        .about(constants::ABOUT)
        .arg(
            Arg::new("command")
                .about("can be one of: show, stop")
                .required(false)
                .index(1),
        )
}
