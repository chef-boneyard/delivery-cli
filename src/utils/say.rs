extern crate term;

use std::time::duration::Duration;
use std::sync::mpsc::{Sender, Receiver};
use std::io::timer::{sleep};
use std::sync::mpsc::channel;
use std::thread::{Thread, JoinGuard};

pub struct Spinner {
    tx: Sender<int>,
    guard: JoinGuard<()>
}

impl Spinner {
    pub fn start() -> Spinner {
        let (tx, rx) = channel::<int>();
        let spinner = Thread::spawn(move|| { Spinner::spin(rx) });
        Spinner{ tx: tx, guard: spinner }
    }

    pub fn stop(self) {
        let _ = self.tx.send(1);
        let _ = self.guard.join();
    }

    fn spin(rx: Receiver<int>) {
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

pub fn say(color: &str, to_say: &str) {
    let mut t = term::stdout().unwrap();
    let color_const = match color {
        "green" => term::color::BRIGHT_GREEN,
        "yellow" => term::color::BRIGHT_YELLOW,
        "red" => term::color::BRIGHT_RED,
        "magenta" => term::color::BRIGHT_MAGENTA,
        "white" => term::color::WHITE,
        _ => term::color::WHITE
    };
    t.fg(color_const).unwrap();
    (write!(t, "{}", to_say)).unwrap();
    t.reset().unwrap()
}

pub fn sayln(color: &str, to_say: &str) {
    let mut t = term::stdout().unwrap();
    say(color, to_say);
    (write!(t, "\n")).unwrap();
}

