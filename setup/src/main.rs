#![feature(concat_idents)]

use clap::Parser;
use convert_case::{Case, Casing};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

use crate::util::{
    ask_user, get_app_exe_path, get_bin_dir, get_config_dir, get_data_dir, get_root_dir,
    read_user_input,
};

mod bytes;
mod util;

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(subcommand)]
    selector: Selector,
}

#[derive(clap::Subcommand, Debug)]
enum Selector {
    Install(InstallOptions),
    Uninstall(UninstallOptions),
}

#[derive(clap::Args, Debug)]
struct InstallOptions {
    /// Your configuration will not be overridden by default, you can reset config to default using
    /// this flag
    #[clap(short, long)]
    override_config: bool,

    #[clap(short, long)]
    enable_autostart: bool,

    #[clap(short, long)]
    interactive: bool,
}

#[derive(clap::Args, Debug)]
struct UninstallOptions {
    /// By default, uninstaller will remove the entire install directory and autostart entry
    /// By enabling this flag you can keep your configuration files untouched
    #[clap(short, long)]
    keep_config: bool,

    #[clap(short, long)]
    interactive: bool,
}

fn main() {
    let options = Options::parse();

    match options.selector {
        Selector::Install(options) => install(options),
        Selector::Uninstall(options) => uninstall(options),
    };
}

fn install_binary_impl(name: &str, bytes: &[u8]) -> std::io::Result<()> {
    let target = get_bin_dir()
        .join(name)
        .with_extension(env::consts::EXE_EXTENSION);
    println!("Copying binary: {}", target.display());

    let mut file = File::create(target)?;
    file.write_all(bytes)?;
    os::set_exe_permissions(&mut file)
}

fn install_config_impl(name: &str, bytes: &[u8], override_config: bool) {
    let target = get_config_dir().join(name).with_extension("lua");

    if override_config || !target.exists() {
        println!("Copying config file: {}", target.display());
        fs::write(&target, bytes).unwrap();
    } else {
        println!(
            "Skipped copying config file (file already exists): {}",
            target.display()
        );
    }
}

macro_rules! install_binary {
    ($name:expr) => {
        install_binary_impl(&stringify!($name).to_case(Case::Snake), $name).unwrap()
    };
}

macro_rules! install_config {
    ($name:expr, $override_config:expr) => {
        install_config_impl(
            &stringify!($name).to_case(Case::Snake),
            $name,
            $override_config,
        )
    };
}

fn install(options: InstallOptions) {
    use bytes::*;

    for directory in &vec![
        get_root_dir(),
        get_bin_dir(),
        get_config_dir(),
        get_data_dir(),
    ] {
        create_dir(directory);
    }

    install_binary!(STEELSERIES_OLED);
    install_binary!(AUDIO);
    install_binary!(CLOCK);
    install_binary!(MEDIA);
    install_binary!(WEATHER);

    let override_config = options.override_config
        || (options.interactive && ask_user("Do you wish to override your config?"));

    install_config!(APPLICATIONS, override_config);
    install_config!(DEVICES, override_config);
    install_config!(SCRIPTS, override_config);
    install_config!(SETTINGS, override_config);

    let autostart = options.enable_autostart
        || (options.interactive
            && ask_user("Do you wish for application to start automatically when logging in?"));

    if autostart {
        autostart_enable();
    }

    run();

    println!("Setup finished, press Enter to exit");
    _ = read_user_input();
}

fn uninstall(options: UninstallOptions) {
    let keep_config =
        options.keep_config || (options.interactive && ask_user("Do you wish to keep config?"));

    autostart_disable();

    let paths = if keep_config {
        vec![get_bin_dir(), get_data_dir()]
    } else {
        vec![get_root_dir()]
    };

    for path in paths {
        println!("Removing {}", path.display());
        fs::remove_dir_all(path).unwrap()
    }
}

fn create_dir(path: &PathBuf) {
    if path.exists() {
        println!("Directory {:?} already exists", path);
    } else {
        println!("Creating directory {:?}", path);
        fs::create_dir_all(path).unwrap();
    }
}

fn autostart_enable() {
    os::autostart_enable();
}

fn autostart_disable() {
    os::autostart_disable();
}

fn run() {
    println!("Running 'Steelseries OLED'");
    Command::new(get_app_exe_path()).spawn().unwrap();
}

#[cfg(target_os = "windows")]
mod os {
    use std::fs::File;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    use crate::util::{get_app_exe_path, get_app_name};

    fn get_registry_entry() -> RegKey {
        let reg_current_user = RegKey::predef(HKEY_CURRENT_USER);
        reg_current_user
            .open_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                KEY_READ | KEY_WRITE,
            )
            .unwrap()
    }

    pub fn autostart_enable() {
        let registry_entry = get_registry_entry();

        let path = get_app_exe_path();
        match registry_entry.set_value(get_app_name(), &path.to_str().unwrap()) {
            Ok(_) => println!("Added 'Steelseries OLED' as an autostart program"),
            Err(err) => println!(
                "Failed to add 'Steelseries OLED' as an autostart program: {}",
                err
            ),
        }
    }

    pub fn autostart_disable() {
        let registry_entry = get_registry_entry();

        match registry_entry.delete_value(get_app_name()) {
            Ok(_) => println!("Removed 'Steelseries OLED' from autostart programs"),
            Err(err) => println!(
                "Failed to remove 'Steelseries OLED' from autostart programs: {}",
                err
            ),
        };
    }

    pub fn set_exe_permissions(_file: &mut File) -> std::io::Result<()> {
        // No special handling required
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod os {
    use std::fs::File;
    use std::os::unix::fs::PermissionsExt;

    pub fn autostart_enable() {
        println!("Autostart setup is not yet available on Linux");
    }

    pub fn autostart_disable() {
        // Nothing to disable yet
    }

    pub fn set_exe_permissions(file: &mut File) -> std::io::Result<()> {
        let mut permissions = file.metadata()?.permissions();
        permissions.set_mode(0o775);
        file.set_permissions(permissions)
    }
}
