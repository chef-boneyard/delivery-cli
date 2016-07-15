//
// Copyright:: Copyright (c) 2015 Chef Software, Inc.
// License:: Apache License, Version 2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use term;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;
use std::thread::{self, JoinHandle};
use std::io::prelude::*;
use std::io;
use std::time::Duration;

/// Because sometimes, you just want a global variable.
static mut show_spinner: bool = true;

pub struct Spinner {
    tx: Sender<isize>,
    guard: JoinHandle<()>
}

impl Spinner {
    pub fn start() -> Spinner {
        let (tx, rx) = channel::<isize>();
        let spinner = thread::spawn(move|| { Spinner::spin(rx) });
        Spinner{ tx: tx, guard: spinner }
    }

    pub fn stop(self) {
        let _ = self.tx.send(1);
        let _ = self.guard.join();
    }

    fn spin(rx: Receiver<isize>) {
        let spinner_chars = vec!["|", "/", "-", "\\"];
        for spin in spinner_chars.iter().cycle() {
            unsafe {
                if show_spinner {
                    say("yellow", *spin);
                }
            }
            let r = rx.try_recv();
            match r {
                Ok(_) => {
                    unsafe {
                        if show_spinner {
                            say("white", "\x08 \x08");
                        }
                    }
                    break;
                },
                Err(_) => {
                    thread::sleep(Duration::from_millis(100));
                    unsafe {
                        if show_spinner {
                            say("white", "\x08");
                        }
                    }
                    continue;
                }
            }
        }
    }
}

pub fn turn_off_spinner() {
    unsafe {
        show_spinner = false;
    }
}

fn say_term(mut t: Box<term::StdoutTerminal>, color: &str, to_say: &str) {
    let color_const = match color {
        "green" => term::color::BRIGHT_GREEN,
        "yellow" => term::color::BRIGHT_YELLOW,
        "red" => term::color::BRIGHT_RED,
        "magenta" => term::color::BRIGHT_MAGENTA,
        "white" => term::color::WHITE,
        "cyan" => term::color::BRIGHT_CYAN,
        _ => term::color::WHITE
    };
    t.fg(color_const).unwrap();
    t.write_all(to_say.as_bytes()).unwrap();
    t.reset().unwrap();
    io::stdout().flush().ok().expect("Could not flush stdout");
}

pub fn say(color: &str, to_say: &str) {
    match term::stdout() {
        Some(t) => say_term(t, color, to_say),
        None => {
            print!("{}", to_say);
            io::stdout().flush().ok().expect("Could not flush stdout");
        }
    }
}

pub fn sayln(color: &str, to_say: &str) {
    say(color, to_say);
    say(color, "\n");
}
