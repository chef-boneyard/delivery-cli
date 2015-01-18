#![allow(unstable)]
extern crate term;

use std::time::duration::Duration;
use std::sync::mpsc::{Sender, Receiver};
use std::io::timer::{sleep};
use std::sync::mpsc::channel;
use std::thread::{Thread, JoinGuard};

pub struct Spinner {
    tx: Sender<isize>,
    guard: JoinGuard<'static ()>
}

impl Spinner {
    pub fn start() -> Spinner {
        let (tx, rx) = channel::<isize>();
        let spinner = Thread::scoped(move|| { Spinner::spin(rx) });
        Spinner{ tx: tx, guard: spinner }
    }

    pub fn stop(self) {
        let _ = self.tx.send(1);
        let _ = self.guard.join();
    }

    fn spin(rx: Receiver<isize>) {
        let spinner_chars = vec!["|", "/", "-", "\\",];
        for spin in spinner_chars.iter().cycle() {
            say("white", "\x08");
            say("yellow", *spin);
            let r = rx.try_recv();
            match r {
                Ok(_) => {
                    say("white", "\x08 \x08");
                    break;
                },
                Err(_) => {
                    sleep(Duration::milliseconds(100i64));
                    continue;
                }
            }
        }
    }
}

fn say_term(mut t: Box<term::Terminal<term::WriterWrapper> + Send>, color: &str, to_say: &str) {
    let color_const = match color {
        "green" => term::color::GREEN,
        "yellow" => term::color::YELLOW,
        "red" => term::color::RED,
        "magenta" => term::color::MAGENTA,
        "white" => term::color::WHITE,
        _ => term::color::WHITE
    };
    t.fg(color_const).unwrap();
    t.write(to_say.as_bytes()).unwrap();
    t.reset().unwrap();
}

pub fn say(color: &str, to_say: &str) {
    match term::stdout() {
        Some(t) => say_term(t, color, to_say),
        None => print!("{}", to_say)
    }
}

pub fn sayln(color: &str, to_say: &str) {
    say(color, to_say);
    say(color, "\n");
}

