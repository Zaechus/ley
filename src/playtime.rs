use std::{fs::OpenOptions, io::Write, time::Duration};

use chrono::{self, SecondsFormat};

use crate::expand_tilde;

pub fn log_playtime(name: &str, seconds: u64) {
    let mut ley_log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(expand_tilde("~/.cache/ley/ley.log"))
        .expect("error appending to ley.log");
    writeln!(
        &mut ley_log,
        "{} Played {} for {}",
        start_time(seconds),
        name,
        format_duration(seconds)
    )
    .unwrap();
}

fn start_time(seconds: u64) -> String {
    (chrono::offset::Local::now()
        - chrono::Duration::from_std(Duration::from_secs(seconds)).unwrap())
    .to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn format_duration(mut seconds: u64) -> String {
    let hours = seconds / 3600;
    seconds -= hours * 3600;
    let minutes = seconds / 60;

    format!(
        "{}{}{}",
        plural(hours, "hour"),
        if hours > 0 && minutes > 0 { " " } else { "" },
        plural(minutes, "minute")
    )
}

fn plural(x: u64, word: &str) -> String {
    match x {
        0 => String::new(),
        1 => format!("1 {}", word),
        _ => format!("{} {}s", &x, word),
    }
}
