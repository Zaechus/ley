use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Duration,
};

use chrono::{self, SecondsFormat};
use toml::{Table, Value};

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

    if !Path::new(&expand_tilde("~/.local/share/ley/data.toml")).exists() {
        eprintln!("~/.local/share/ley/data.toml does not exist. Skipping log_playtime...");
        return;
    }

    let mut data_toml = fs::read_to_string(expand_tilde("~/.local/share/ley/data.toml"))
        .unwrap_or_default()
        .parse::<Table>()
        .expect("not a valid toml file");
    if let Some(t) = data_toml.get_mut(name) {
        if let Some(v) = t.get_mut("playtime") {
            *v = Value::Integer(v.as_integer().unwrap() + seconds as i64)
        } else {
            t.as_table_mut()
                .unwrap()
                .insert("playtime".to_string(), Value::Integer(seconds as i64));
        }
    } else {
        let mut t = Table::new();
        t.insert("playtime".to_string(), Value::Integer(seconds as i64));
        data_toml.insert(name.to_string(), Value::Table(t));
    };

    fs::write(
        expand_tilde("~/.local/share/ley/data.toml"),
        data_toml.to_string(),
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
