extern crate term;

use std::time::duration::Duration;
use std::io::timer::{sleep};

pub fn spinner(rx: Receiver<int>, tx: Sender<int>) {
    let spinner = vec!["|", "/", "-", "\\",];
    for spin in spinner.iter().cycle() {
        say("white", "\x08");
        say("yellow", *spin);
        let r = rx.try_recv();
        match r {
            Ok(_) => {
                say("white", "\x08 ");
                tx.send(1);
                break;
            },
            Err(_) => {
                sleep(Duration::milliseconds(100i64));
                continue;
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

