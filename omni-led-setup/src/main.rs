use clap::{ArgAction, Args, Parser, Subcommand};
use convert_case::{Case, Casing};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::{env, fs};

use crate::util::{
    ask_user, get_app_exe_path, get_app_name, get_bin_dir, get_config_dir, get_data_dir,
    get_root_dir, read_user_input,
};

mod bytes;
mod util;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(subcommand)]
    selector: Option<Selector>,
}

#[derive(Subcommand, Debug)]
enum Selector {
    Install(InstallOptions),
    Uninstall(UninstallOptions),
}

#[derive(Args, Debug)]
struct InstallOptions {
    /// Run in interactive mode. Installer will prompt the user for
    /// responses instead of getting settings from CLI options.
    #[clap(short, long)]
    interactive: bool,

    /// Override your config files with defaults. Required in non-interactive mode.
    #[clap(
        short, long, action = ArgAction::Set,
        conflicts_with = "interactive", required_unless_present = "interactive"
    )]
    override_config: Option<bool>,

    /// Control if installer should make program start on login. Required in non-interactive mode.
    #[clap(
        short, long, action = ArgAction::Set,
        conflicts_with = "interactive", required_unless_present = "interactive"
    )]
    enable_autostart: Option<bool>,
}

#[derive(Args, Debug)]
struct UninstallOptions {
    /// Run in interactive mode. Installer will prompt the user for
    /// responses instead of getting settings from CLI options.
    #[clap(short, long)]
    interactive: bool,

    /// Override your config files with defaults. Required in non-interactive mode.
    #[clap(
        short, long, action = ArgAction::Set,
        conflicts_with = "interactive", required_unless_present = "interactive"
    )]
    keep_config: Option<bool>,
}

fn main() {
    let options = Options::parse();

    match options.selector {
        Some(Selector::Install(options)) => install(options),
        Some(Selector::Uninstall(options)) => uninstall(options),
        None => select_action(),
    };
}

fn install_license() {
    let target = get_root_dir().join("LICENSE");
    println!("Copying license file: {}", target.display());
    fs::write(&target, bytes::LICENSE).unwrap();
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

fn select_action() {
    let action = loop {
        println!("Select action to perform:");
        println!("1. Install");
        println!("2. Uninstall");
        println!("3. Exit");

        let response = read_user_input();
        let response = response.trim();
        match response.parse::<usize>() {
            Ok(r @ 1..=3) => break r,
            _ => {}
        }
    };

    match action {
        1 => install(InstallOptions {
            interactive: true,
            override_config: None,
            enable_autostart: None,
        }),
        2 => uninstall(UninstallOptions {
            interactive: true,
            keep_config: None,
        }),
        _ => std::process::exit(0),
    }
}

fn install(options: InstallOptions) {
    use bytes::*;

    for directory in &vec![
        get_root_dir(),
        get_bin_dir(),
        get_config_dir(),
        get_data_dir(),
    ] {
        fs::create_dir_all(directory).unwrap();
    }

    install_license();

    install_binary!(OMNI_LED);
    install_binary!(AUDIO);
    install_binary!(CLOCK);
    install_binary!(IMAGES);
    install_binary!(MEDIA);
    install_binary!(WEATHER);

    let override_config = options.override_config == Some(true)
        || (options.interactive && ask_user("Do you wish to override your config?"));

    install_config!(APPLICATIONS, override_config);
    install_config!(DEVICES, override_config);
    install_config!(SCRIPTS, override_config);
    install_config!(SETTINGS, override_config);

    let autostart = options.enable_autostart == Some(true)
        || (options.interactive
            && ask_user("Do you wish for application to start automatically when logging in?"));

    if autostart {
        os::autostart_enable();
    }

    run();

    println!("Setup finished, press Enter to exit");
    _ = read_user_input();
}

fn uninstall(options: UninstallOptions) {
    let keep_config = options.keep_config == Some(true)
        || (options.interactive && ask_user("Do you wish to keep config?"));

    os::autostart_disable();

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

fn run() {
    println!("Running '{}'", get_app_name());
    Command::new(get_app_exe_path()).spawn().unwrap();
}

#[cfg(target_os = "windows")]
mod os {
    use std::fs::File;
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};

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
            Ok(_) => println!("Added '{}' as an autostart program", get_app_name()),
            Err(err) => println!(
                "Failed to add '{}' as an autostart program: {}",
                get_app_name(),
                err
            ),
        }
    }

    pub fn autostart_disable() {
        let registry_entry = get_registry_entry();

        match registry_entry.delete_value(get_app_name()) {
            Ok(_) => println!("Removed '{}' from autostart programs", get_app_name()),
            Err(err) => println!(
                "Failed to remove '{}' from autostart programs: {}",
                get_app_name(),
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

#[cfg(target_os = "macos")]
mod os {
    use std::fs::File;

    pub fn autostart_enable() {
        todo!()
    }

    pub fn autostart_disable() {
        todo!()
    }

    pub fn set_exe_permissions(_file: &mut File) -> std::io::Result<()> {
        todo!()
    }
}
