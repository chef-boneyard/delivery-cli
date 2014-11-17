extern crate term;

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

