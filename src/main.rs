use std::{env, fs, path::Path, process::Command};

use toml::{Table, Value};

fn main() {
    let args: Vec<_> = env::args().collect();
    let home = env::var("HOME").unwrap();
    let sway = env::var("XDG_CURRENT_DESKTOP").as_deref() == Ok("sway");

    let config = fs::read_to_string(expand_tilde("~/.config/ley/ley.toml", &home))
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

    let mut command = Vec::new();

    let runner = if let Some(Value::String(runner)) = game.get("runner") {
        command.push(runner.clone());
        runner.to_owned()
    } else {
        String::new()
    };

    if let Some(Value::String(val)) = game.get("wine") {
        let val = expand_tilde(val, &home);
        env::set_var("WINE", &val);
        command.push(val);
    }

    if let Some(Value::String(val)) = game.get("prefix") {
        let val = expand_tilde(val, &home);
        env::set_var("WINEPREFIX", &val);

        if !Path::new(&val).exists() {
            if runner.is_empty() {
                Command::new("winetricks").arg("dxvk").status().unwrap();
            } else {
                Command::new(runner)
                    .args(["winetricks", "dxvk"])
                    .status()
                    .unwrap();
            }
        }
    }

    if let Some(Value::String(exe)) = game.get("exe") {
        let exe = expand_tilde(exe, &home);

        let path = exe
            .strip_suffix(Path::new(&exe).file_name().unwrap().to_str().unwrap())
            .unwrap();
        env::set_current_dir(path).unwrap();

        command.push(exe);
    }

    if let Some(Value::String(dir)) = game.get("dir") {
        env::set_current_dir(expand_tilde(dir, &home)).unwrap();
    }

    if let Some(Value::Array(val)) = game.get("args") {
        command.extend(val.iter().map(|v| v.as_str().unwrap().to_owned()));
    }

    Command::new(&command[0])
        .args(&command[1..])
        .spawn()
        .unwrap();
}

fn expand_tilde(s: &str, home: &str) -> String {
    if s.starts_with("~/") {
        format!("{}/{}", home, s.strip_prefix("~/").unwrap())
    } else {
        s.to_owned()
    }
}
