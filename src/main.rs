use std::{
    env,
    fs::{self, File},
    path::Path,
    process::Command,
};

use toml::{Table, Value};

use ley::expand_tilde;

fn main() {
    let args: Vec<_> = env::args().collect();
    let home = env::var("HOME").unwrap();
    let sway = env::var("XDG_CURRENT_DESKTOP").as_deref() == Ok("sway");

    let config = fs::read_to_string(expand_tilde("~/.config/ley/ley.toml", &home))
        .expect("error reading config file")
        .parse::<Table>()
        .expect("not a valid toml file");
    let game = if let Some(game_id) = args.get(1) {
        config.get(game_id).expect("game not found")
    } else {
        for game in config.keys().collect::<Vec<_>>() {
            println!("{game}");
        }
        return;
    };

    fs::create_dir_all(expand_tilde("~/.cache/ley/", &home))
        .expect("error creating cache directory");
    let stdout_log = File::create(expand_tilde("~/.cache/ley/stdout.log", &home))
        .expect("error creating log file");
    let stderr_log = File::create(expand_tilde("~/.cache/ley/stderr.log", &home))
        .expect("error creating log file");

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

    env::set_var("WINE", "wine");

    if !cfg!(debug_assertions) {
        env::set_var("WINEDEBUG", "-all");
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

    let mut command = Vec::new();

    let pre = if let Some(Value::String(pre)) = game.get("pre") {
        command.push(pre.clone());
        pre.to_owned()
    } else {
        String::new()
    };

    if let Some(Value::String(runner)) = game.get("runner") {
        let val = expand_tilde(runner, &home);
        env::set_var("WINE", &val);
        command.push(val);
    }

    if let Some(Value::String(val)) = game.get("prefix") {
        env::set_var("WINEPREFIX", expand_tilde(val, &home));
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
        command.extend(val.iter().map(|v| expand_tilde(v.as_str().unwrap(), &home)));
    }

    if args.len() > 2 {
        if args[2] == "setup" {
            let command = if pre.is_empty() {
                vec!["winetricks", "dxvk"]
            } else {
                vec![pre.as_str(), "winetricks", "dxvk"]
            };

            Command::new(command[0])
                .args(&command[1..])
                .status()
                .unwrap();
        } else {
            let mut command = if pre.is_empty() {
                Vec::new()
            } else {
                vec![pre]
            };

            if args[2] == "winecfg" {
                command.push(env::var("WINE").unwrap());
                command.push("winecfg".to_owned())
            } else {
                command.extend_from_slice(&args[2..])
            }

            if command.len() == 1 {
                Command::new(&command[0]).status().unwrap();
            } else {
                Command::new(&command[0])
                    .args(&command[1..])
                    .status()
                    .unwrap();
            }
        }
    } else {
        Command::new(&command[0])
            .args(&command[1..])
            .stdout(stdout_log)
            .stderr(stderr_log)
            .spawn()
            .unwrap();
    }
}
