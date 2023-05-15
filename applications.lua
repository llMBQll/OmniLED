local APPLICATIONS_PATH = 'C:\\dev\\rust\\steelseries_oled_applications\\target\\release\\'

load_app {
    path = APPLICATIONS_PATH .. 'clock.exe',
    args = { SERVER.address },
}

--load_app {
--    path = APPLICATIONS_PATH .. 'audio.exe',
--    args = { SERVER.address },
--}
--
--load_app {
--    path = APPLICATIONS_PATH .. 'spotify.exe',
--    args = { SERVER.address },
--}
