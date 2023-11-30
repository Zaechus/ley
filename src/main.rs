use std::{env, fs, process::Command};

use toml::{Table, Value};

fn main() {
    let args: Vec<_> = env::args().collect();
    let home = env::var("HOME").unwrap();

    let config = fs::read_to_string("~/.config/ley/ley.toml".replace('~', &home))
        .expect("error reading config file")
        .parse::<Table>()
        .expect("not a valid toml file");
    let game = config
        .get(args.get(1).expect("expected a game id"))
        .expect("game not found");

    if let Some(Value::String(val)) = game.get("prefix") {
        env::set_var("WINEPREFIX", val.replace('~', &home));
    }

    if let Some(Value::String(val)) = game.get("arch") {
        if val == "win32" {
            env::set_var("WINEARCH", "win32");
        } else {
            env::set_var("WINEARCH", "win64");
        }
    } else {
        env::set_var("WINEARCH", "win64");
    }

    if let Some(Value::Boolean(val)) = game.get("esync") {
        env::set_var("WINEESYNC", if *val { "1" } else { "0" });
    } else {
        env::set_var("WINEESYNC", "1");
    }

    if let Some(Value::String(exe)) = game.get("exe") {
        let exe = exe.replace('~', &home);

        let mut game_args = vec![exe.as_str()];

        if let Some(Value::Array(val)) = game.get("args") {
            game_args.extend(val.iter().map(|v| v.as_str().unwrap()));
        }

        Command::new("wine").args(game_args).spawn().unwrap();
    }
}
