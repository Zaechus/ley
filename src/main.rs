use std::{
    env,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    thread,
    time::{Duration, Instant},
};

use clap::Parser;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind};
use toml::{Table, Value};

use ley::{expand_tilde, log_playtime};

// TODO: exclusive params
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Game name
    name: Option<String>,

    /// Install an exe with wine
    #[arg(long)]
    install: Option<PathBuf>,

    /// Run setup commands in a wine prefix
    #[arg(long)]
    setup: bool,

    /// Run winecfg in a wine prefix
    #[arg(long)]
    winecfg: bool,

    /// Prefer changing output scale over resolution
    #[arg(long)]
    scale: bool,

    /// Run a command within a game configuration
    #[arg(raw = true)]
    command: Vec<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if !Path::new(&expand_tilde("~/.config/ley/ley.toml")).exists() {
        eprintln!("~/.config/ley/ley.toml does not exist...");
        return ExitCode::FAILURE;
    }

    let config = fs::read_to_string(expand_tilde("~/.config/ley/ley.toml"))
        .expect("error reading config file")
        .parse::<Table>()
        .expect("not a valid toml file");
    let name = if let Some(name) = cli.name.as_deref() {
        name
    } else {
        for (name, game) in config {
            let game_installed = if let Some(Value::String(dir)) = game.get("dir") {
                Path::new(&expand_tilde(dir)).is_dir()
            } else if let Some(Value::String(exe)) = game.get("exe") {
                Path::new(&expand_tilde(exe)).exists()
            } else {
                false
            };

            if game_installed {
                println!("{name}"); // TODO: installed size and playtime as a table
            }
        }
        return ExitCode::SUCCESS;
    };
    let game = if let Some(v) = config.get(name) {
        v
    } else {
        eprintln!("game not found");
        return ExitCode::FAILURE;
    };

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

    if let Some(Value::String(val)) = game.get("prefix") {
        env::set_var("WINEPREFIX", expand_tilde(val));
    }

    let mut command = Vec::new();

    let pre = if let Some(Value::String(pre)) = game.get("pre") {
        command.push(pre.clone());
        pre.to_owned()
    } else {
        String::new()
    };

    if let Some(Value::String(runner)) = game.get("runner") {
        let val = expand_tilde(runner);
        env::set_var("WINE", &val);
        command.push(val);
    }

    if let Some(Value::String(exe)) = game.get("exe") {
        let exe = expand_tilde(exe);

        let path = exe
            .strip_suffix(Path::new(&exe).file_name().unwrap().to_str().unwrap())
            .unwrap();
        env::set_current_dir(path).ok();

        command.push(exe);
    }

    // dir option overrides exe path
    if let Some(Value::String(dir)) = game.get("dir") {
        env::set_current_dir(expand_tilde(dir)).unwrap();
    }

    if let Some(Value::Array(val)) = game.get("args") {
        command.extend(val.iter().map(|v| expand_tilde(v.as_str().unwrap())));
    }

    if let Some(Value::Table(vars)) = game.get("env") {
        for (k, v) in vars {
            if let Value::Integer(i) = v {
                env::set_var(k, i.to_string());
            } else {
                env::set_var(k, v.as_str().unwrap());
            }
        }
    };

    let run_with_wine = |cmd: &str| {
        let wine = if let Ok(s) = env::var("WINE") {
            s
        } else {
            "wine".to_owned()
        };
        let command = if pre.is_empty() {
            vec![&wine, cmd]
        } else {
            vec![&pre, &wine, cmd]
        };

        Command::new(command[0])
            .args(&command[1..])
            .status()
            .unwrap();
    };

    if !cli.command.is_empty() {
        Command::new(&cli.command[0])
            .args(&cli.command[1..])
            .status()
            .unwrap();
    } else if let Some(install) = cli.install {
        run_with_wine(&expand_tilde(
            &install.into_os_string().into_string().unwrap(),
        ));
    } else if cli.setup {
        let mut command = if pre.is_empty() {
            vec!["winetricks", "-q", "dxvk"]
        } else {
            vec![pre.as_str(), "winetricks", "-q", "dxvk"]
        };

        if let Some(Value::Array(val)) = game.get("winetricks") {
            command.extend(val.iter().map(|v| v.as_str().unwrap()));
        }

        Command::new(command[0])
            .args(&command[1..])
            .status()
            .unwrap();
    } else if cli.winecfg {
        run_with_wine("winecfg");
    } else {
        if env::var("XDG_CURRENT_DESKTOP").as_deref() == Ok("sway") {
            match (
                game.get("res").is_some(),
                game.get("scale").is_some(),
                cli.scale,
            ) {
                (_, true, true) | (false, true, _) => {
                    if let Some(Value::String(scale)) = game.get("scale") {
                        Command::new("swaymsg")
                            .args(["output", "-", "scale", scale])
                            .spawn()
                            .unwrap();
                    }
                }
                _ => {
                    if let Some(Value::String(res)) = game.get("res") {
                        Command::new("swaymsg")
                            .args(["output", "-", "mode", res])
                            .spawn()
                            .unwrap();
                    }
                }
            }

            if let Some(Value::String(accel)) = game.get("mouse_speed") {
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

        fs::create_dir_all(expand_tilde("~/.cache/ley/")).expect("error creating cache directory");
        let stdout_log =
            File::create(expand_tilde("~/.cache/ley/stdout.log")).expect("error creating log file");
        let stderr_log =
            File::create(expand_tilde("~/.cache/ley/stderr.log")).expect("error creating log file");
        let now = Instant::now();

        if File::create_new(expand_tilde(&format!("~/.cache/ley/{}.lck", name))).is_err() {
            eprintln!("{} is already running.", name);
            return ExitCode::FAILURE;
        }

        Command::new(&command[0])
            .args(&command[1..])
            .stdout(stdout_log)
            .stderr(stderr_log)
            .status()
            .unwrap();

        // some Windows executables are launchers, so additionally track the winedevice.exe pid
        if let Some(Value::String(prefix)) = game.get("prefix") {
            let mut sys = System::new_with_specifics(
                RefreshKind::new()
                    .with_processes(ProcessRefreshKind::new().with_cwd(UpdateKind::OnlyIfNotSet)),
            );
            if let Some((winedevice_pid, _)) = sys.processes().iter().find(|(_, process)| {
                if let Some(cwd) = process.cwd() {
                    process.name() == "winedevice.exe"
                        && cwd
                            .to_str()
                            .unwrap_or_default()
                            .contains(&expand_tilde(prefix))
                } else {
                    false
                }
            }) {
                let pid = *winedevice_pid;
                loop {
                    if sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true) == 0 {
                        break;
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }

        _ = fs::remove_file(expand_tilde(&format!("~/.cache/ley/{}.lck", name)));

        let seconds = now.elapsed().as_secs();
        if seconds > 119 {
            log_playtime(name, seconds);
        }
    }

    ExitCode::SUCCESS
}
