use std::env;
use std::path::PathBuf;

fn main() {
    let root = dirs_next::config_dir().expect("Couldn't get default config directory");
    let root = root.join("SteelseriesOLED");
    let applications_root = root.join("applications");
    create_dir(&root);
    create_dir(&applications_root);

    copy("", &root, "screens.lua");
    for file in vec!["applications.lua", "scripts.lua", "settings.lua"] {
        create_file(&root, file);
    }

    for app in vec!["audio", "clock", "media", "weather"] {
        copy("target/release/", &applications_root, &get_bin_path(app));
    }
    copy("target/release/", &root, &get_bin_path("steelseries_oled"));
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

fn copy(src_root: &str, dst_root: &PathBuf, name: &str) {
    let src = PathBuf::from(src_root).join(name);
    let dst = PathBuf::from(dst_root).join(name);
    println!("Copying {:?} to {:?}", src, dst);
    std::fs::copy(src, dst).expect("Failed to copy files");
}

fn create_file(root: &PathBuf, name: &str) {
    let path = root.join(name);
    if path.exists() {
        println!("File {:?} already exists", path);
    } else {
        println!("Creating file {:?}", path);
        std::fs::write(path, "-- You can put your contents here").unwrap();
    }
}
