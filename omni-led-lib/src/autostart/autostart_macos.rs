use std::path::PathBuf;

use crate::autostart::AutoStartInterface;
use crate::constants::constants::Constants;

pub struct AutoStart;

impl AutoStart {
    fn autostart_file_path() -> mlua::Result<PathBuf> {
        let home =
            dirs_next::home_dir().ok_or(mlua::Error::runtime("Failed to get home directory"))?;
        Ok(home.join("Library/LaunchAgents/io.github.llMBQll.OmniLED.plist"))
    }

    fn generate_plist_file_content() -> String {
        let exe_path = Constants::current_exe();

        let mut plist = String::new();
        plist += "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
        plist += "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n";
        plist += "<plist version=\"1.0\">\n";
        plist += "<dict>\n";
        plist += "    <key>Label</key>\n";
        plist += "    <string>io.github.llMBQll.OmniLED</string>\n";
        plist += "    <key>ProgramArguments</key>\n";
        plist += "    <array>\n";
        plist += &format!("    <string>{}</string>\n", exe_path.display());
        plist += "    </array>\n";
        plist += "    <key>RunAtLoad</key><true/>\n";
        plist += "</dict>\n";
        plist += "</plist>\n";
        plist
    }
}

impl AutoStartInterface for AutoStart {
    fn enable() -> mlua::Result<()> {
        let path = Self::autostart_file_path()?;
        std::fs::write(path, Self::generate_plist_file_content())?;
        Ok(())
    }

    fn disable() -> mlua::Result<()> {
        let path = Self::autostart_file_path()?;
        std::fs::remove_file(path)?;
        Ok(())
    }

    fn enabled() -> mlua::Result<bool> {
        let path = Self::autostart_file_path()?;
        let exists = std::fs::exists(path)?;
        Ok(exists)
    }
}
