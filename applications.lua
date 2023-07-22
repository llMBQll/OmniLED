local function make_path(path)
    if PLATFORM.os == 'windows' then
        APPLICATIONS_PATH = 'C:\\dev\\rust\\steelseries_oled_applications\\target\\release\\'
        EXTENSION = '.exe'

        return APPLICATIONS_PATH .. path .. EXTENSION
    elseif PLATFORM.os == 'linux' then
        APPLICATIONS_PATH = '/home/mbq/dev/rust/steelseries_oled_applications/target/release/'
        EXTENSION = ''

        return APPLICATIONS_PATH .. path .. EXTENSION
    end

    return ''
end

load_app {
    path = make_path('clock'),
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