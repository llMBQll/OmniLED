local APPLICATIONS_PATH = '/home/mbq/dev/rust/steelseries_oled_applications/target/release/'

load_app {
    path = APPLICATIONS_PATH .. 'clock',
    args = { SERVER.address },
}

-- load_app {
--     path = APPLICATIONS_PATH .. 'audio.exe',
--     args = { SERVER.address },
-- }

-- load_app {
--     path = APPLICATIONS_PATH .. 'spotify.exe',
--     args = { SERVER.address },
-- }
