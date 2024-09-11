use std::env;
use std::path::PathBuf;

pub fn get_app_name() -> &'static str {
    "SteelseriesOLED"
}

fn get_exe_name() -> &'static str {
    "steelseries_oled"
}

pub fn get_root_dir() -> PathBuf {
    let root = dirs_next::config_dir().expect("Couldn't get default config directory");
    let root = root.join(get_app_name());
    root
}

pub fn get_bin_dir() -> PathBuf {
    let root = get_root_dir();
    root.join("bin")
}

pub fn get_config_dir() -> PathBuf {
    let root = get_root_dir();
    root.join("config")
}

pub fn get_data_dir() -> PathBuf {
    let root = get_root_dir();
    root.join("data")
}

pub fn get_app_exe_path() -> PathBuf {
    get_bin_dir()
        .join(get_exe_name())
        .with_extension(env::consts::EXE_EXTENSION)
}

pub fn ask_user(message: &str) -> bool {
    println!("{message} [Y/N]");

    let response = read_user_input();
    let response = response.trim().to_lowercase();
    if response != "y" && response != "n" {
        println!("Please choose [Y/N]");
        return ask_user(message);
    }

    response == "y"
}

pub fn read_user_input() -> String {
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
