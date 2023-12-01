use std::{env, fs, process::Command};

use toml::{Table, Value};

fn main() {
    let args: Vec<_> = env::args().collect();
    let home = env::var("HOME").unwrap();
    let sway = env::var("XDG_CURRENT_DESKTOP").as_deref() == Ok("sway");

    let config = fs::read_to_string("~/.config/ley/ley.toml".replace('~', &home))
        .expect("error reading config file")
        .parse::<Table>()
        .expect("not a valid toml file");
    let game = config
        .get(args.get(1).expect("expected a game id"))
        .expect("game not found");

    if let Some(Value::String(res)) = game.get("res") {
        if sway {
            Command::new("swaymsg")
                .args(["output", "-", "mode", res])
                .spawn()
                .unwrap();
        }
    }

    if let Some(Value::String(dir)) = game.get("dir") {
        env::set_current_dir(dir.replace('~', &home)).unwrap();
    }

    if let Some(Value::String(accel)) = game.get("mouse_speed") {
        if sway {
            Command::new("swaymsg")
                .args([
                    "input",
                    "type:pointer",
                    "pointer_accel",
                    &format!("'{}'", accel),
                ])
                .spawn()
                .unwrap();
        }
    }

    env::set_var("WINEDEBUG", "-all");

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

    let runner = if let Some(Value::String(runner)) = game.get("runner") {
        runner
    } else {
        "wine"
    };

    let mut game_args = Vec::new();

    if let Some(Value::String(exe)) = game.get("exe") {
        game_args.push(exe.replace('~', &home));
    }

    if let Some(Value::String(val)) = game.get("args") {
        game_args.extend(val.split_whitespace().map(str::to_owned));
    } else if let Some(Value::Array(val)) = game.get("args") {
        game_args.extend(val.iter().map(|v| v.to_string()));
    }

    Command::new(runner).args(game_args).spawn().unwrap();
}
