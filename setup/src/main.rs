use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let options = Options::parse();
    let source = match (options.debug, options.release) {
        (Some(true), _) => "target/debug/",
        _ => "target/release/",
    };

    let root = get_root();
    let data_root = root.join("data");
    let config_root = root.join("config");
    let applications_root = root.join("applications");
    create_dir(&root);
    create_dir(&data_root);
    create_dir(&config_root);
    create_dir(&applications_root);

    let settings_source_root = if options.dev { "config" } else { "defaults" };
    for file in vec![
        "applications.lua",
        "screens.lua",
        "scripts.lua",
        "settings.lua",
    ] {
        copy(&settings_source_root, &config_root, file, false);
    }

    for app in vec!["audio", "clock", "media", "weather"] {
        copy(source, &applications_root, &get_bin_path(app), true);
    }
    copy(source, &root, &get_bin_path("steelseries_oled"), true);

    setup_autostart();
    start();

    println!("Setup finished, press Enter to exit");
    _ = read_user_input();
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
struct Options {
    /// Install debug binaries
    #[clap(long, group = "type", action = clap::ArgAction::SetTrue)]
    debug: Option<bool>,

    /// Install release binaries, this is the default behavior
    #[clap(long, group = "type", action = clap::ArgAction::SetTrue)]
    release: Option<bool>,

    /// Run in dev environment
    #[clap(short, long, group = "type")]
    dev: bool,
}

fn get_app_name() -> &'static str {
    "SteelseriesOLED"
}

fn get_root() -> PathBuf {
    let root = dirs_next::config_dir().expect("Couldn't get default config directory");
    let root = root.join(get_app_name());
    root
}

fn create_dir(path: &PathBuf) {
    if path.exists() {
        println!("Directory {:?} already exists", path);
    } else {
        println!("Creating directory {:?}", path);
        std::fs::create_dir_all(path).unwrap();
    }
}

fn get_bin_path(name: &str) -> String {
    PathBuf::from(name)
        .with_extension(env::consts::EXE_EXTENSION)
        .to_str()
        .unwrap()
        .to_string()
}

fn copy(src_root: &str, dst_root: &PathBuf, name: &str, override_file: bool) {
    let src = PathBuf::from(src_root).join(name);
    let dst = PathBuf::from(dst_root).join(name);

    if !src.exists() {
        eprintln!("Source {:?} doesn't exist", src);
        std::process::exit(1);
    }

    if dst.exists() {
        if override_file {
            println!("Copying {:?} to {:?} [Override!]", src, dst);
            std::fs::copy(src, dst).expect("Failed to copy files");
        } else {
            println!(
                "Copying {:?} to {:?} skipped, file already exists",
                src, dst
            );
        }
    } else {
        println!("Copying {:?} to {:?}", src, dst);
        std::fs::copy(src, dst).expect("Failed to copy files");
    }
}

fn read_user_input() -> String {
    let mut response = String::new();
    std::io::stdin().read_line(&mut response).unwrap();

    if let Some('\n') = response.chars().next_back() {
        response.pop();
    }
    if let Some('\r') = response.chars().next_back() {
        response.pop();
    }

    response
}

#[cfg(target_os = "windows")]
fn setup_autostart() {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    println!("Do you want 'Steelseries OLED' to launch automatically when you log in [Y/N]?");
    let response = read_user_input();
    let autostart = response.to_ascii_lowercase() == "y";

    let reg_current_user = RegKey::predef(HKEY_CURRENT_USER);
    let reg_run = reg_current_user
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            KEY_READ | KEY_WRITE,
        )
        .unwrap();

    if autostart {
        let path = get_root().join("steelseries_oled.exe");
        match reg_run.set_value(get_app_name(), &path.to_str().unwrap()) {
            Ok(_) => println!("Added 'Steelseries OLED' as an autostart program"),
            Err(err) => println!(
                "Failed to add 'Steelseries OLED' as an autostart program: {}",
                err
            ),
        }
    } else {
        _ = reg_run.delete_value(get_app_name());
        println!("'Steelseries OLED' will not start automatically");
    }
}

#[cfg(target_os = "linux")]
fn setup_autostart() {
    println!("Autostart setup is not yet available on Linux");
}

fn start() {
    println!("Do you want to start 'Steelseries OLED' now [Y/N]?");
    let response = read_user_input();
    if response.to_ascii_lowercase() != "y" {
        return;
    }

    let path = get_root()
        .join(PathBuf::from("steelseries_oled").with_extension(env::consts::EXE_EXTENSION));
    println!("Running 'Steelseries OLED'");
    Command::new(path).spawn().unwrap();
}
