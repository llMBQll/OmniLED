local function volume()
    return {
        data = {
            Text {
                text = AUDIO.IsMuted and 'Muted' or AUDIO.Volume,
                position = Rectangle {
                    origin = Point { x = 0, y = 0 },
                    size = Size { width = SCREEN.Width, height = SCREEN.Height / 2 },
                },
                modifiers = Modifiers { font_size = 27 },
            },
            Text {
                text = AUDIO.Name,
                position = Rectangle {
                    origin = Point { x = 0, y = SCREEN.Height / 2 },
                    size = Size { width = SCREEN.Width, height = SCREEN.Height / 2 - 4 },
                },
                modifiers = Modifiers { scrolling = true },
            },
        },
        duration = 2000,
        repeats = 'once',
    }
end

local SPOTIFY_DURATION = PLATFORM.Os == 'windows' and 5000 or 1000
local function spotify()
    return {
        data = {
            Bar {
                value = SPOTIFY.Progress * 100.0 / SPOTIFY.Duration,
                position = Rectangle {
                    origin = Point { x = 0, y = 0 },
                    size = Size { width = SCREEN.Width, height = 2 },
                },
            },
            Text {
                text = string.format("%s - %s", SPOTIFY.Artist, SPOTIFY.Title),
                position = Rectangle {
                    origin = Point { x = 0, y = 1 },
                    size = Size { width = SCREEN.Width, height = 16 },
                },
                modifiers = Modifiers { scrolling = true },
            },
            Text {
                text = string.format("%02d:%02d", CLOCK.Hours, CLOCK.Minutes),
                position = Rectangle {
                    origin = Point { x = 0, y = SCREEN.Height - 16 },
                    size = Size { width = 50, height = 12 },
                },
                modifiers = Modifiers { font_size = 16 },
            },
            Text {
                text = string.format("%.3s%02d", CLOCK.MonthNames[CLOCK.Month + 1], CLOCK.MonthDay),
                position = Rectangle {
                    origin = Point { x = SCREEN.Width - 50, y = SCREEN.Height - 16 },
                    size = Size { width = 50, height = 12 },
                },
                modifiers = Modifiers { font_size = 16 },
            },
            Bar {
                value = CLOCK.Seconds * 100.0 / 59,
                position = Rectangle {
                    origin = Point { x = 0, y = SCREEN.Height - 2 },
                    size = Size { width = SCREEN.Width, height = 2 },
                },
            },
        },
        duration = SPOTIFY_DURATION,
        repeats = 'to_fit',
    }
end

local function clock()
    return {
        data = {
            Text {
                text = string.format("%02d", CLOCK.Hours),
                position = Rectangle {
                    origin = Point { x = 10, y = 1 },
                    size = Size { width = 54, height = 35 },
                },
                modifiers = Modifiers { font_size = 47 },
            },
            Text {
                text = string.format("%02d", CLOCK.Minutes),
                position = Rectangle {
                    origin = Point { x = 64, y = 0 },
                    size = Size { width = 54, height = 26 },
                },
                modifiers = Modifiers { font_size = 36 },
            },
            Text {
                text = string.format("%.3s%02d", CLOCK.MonthNames[CLOCK.Month + 1], CLOCK.MonthDay),
                position = Rectangle {
                    origin = Point { x = 66, y = 27 },
                    size = Size { width = 54, height = 10 },
                },
                modifiers = Modifiers { font_size = 14 },
            },
            Bar {
                value = CLOCK.Seconds * 100.0 / 59,
                position = Rectangle {
                    origin = Point { x = 0, y = SCREEN.Height - 2 },
                    size = Size { width = SCREEN.Width, height = 2 },
                },
            }
        },
        duration = 1000,
    }
end

local function weather()
    return {
        data = {
            Text {
                text = string.format("%.1f°C %.1fkm/h", WEATHER.Temperature, WEATHER.WindSpeed),
                position = Rectangle {
                    origin = Point { x = 0, y = 0 },
                    size = Size { width = SCREEN.Width, height = 16 },
                },
                modifiers = Modifiers { font_size = 22, scrolling = true },
            },
            Text {
                text = string.format("%02d:%02d", CLOCK.Hours, CLOCK.Minutes),
                position = Rectangle {
                    origin = Point { x = 0, y = SCREEN.Height - 16 },
                    size = Size { width = 50, height = 12 },
                },
                modifiers = Modifiers { font_size = 16 },
            },
            Text {
                text = string.format("%.3s%02d", CLOCK.MonthNames[CLOCK.Month + 1], CLOCK.MonthDay),
                position = Rectangle {
                    origin = Point { x = SCREEN.Width - 50, y = SCREEN.Height - 16 },
                    size = Size { width = 50, height = 12 },
                },
                modifiers = Modifiers { font_size = 16 },
            },
            Bar {
                value = CLOCK.Seconds * 100.0 / 59,
                position = Rectangle {
                    origin = Point { x = 0, y = SCREEN.Height - 2 },
                    size = Size { width = SCREEN.Width, height = 2 },
                },
            },
        },
        duration = 1000,
        repeats = 'once',
    }
end

local current_screen = 1
local max_screens = 2
SHORTCUTS:register(
        { 'KEY(RAlt)', 'KEY(Slash)' },
        function()
            if current_screen == max_screens then
                current_screen = 1
            else
                current_screen = current_screen + 1
            end
            LOG:debug('Current screen: ' .. current_screen)
        end,
        SHORTCUTS.RESET_STATE
)

function current_screen_is(screen)
    return function()
        return current_screen == screen
    end
end

local volume_script = {
    action = volume,
    predicate = current_screen_is(1),
    run_on = { 'AUDIO.IsMuted', 'AUDIO.Name', 'AUDIO.Volume' },
}

local spotify_script = {
    action = spotify,
    predicate = current_screen_is(1),
    run_on = { 'SPOTIFY.Artist', 'SPOTIFY.Progress', 'SPOTIFY.Title' },
}

local clock_script = {
    action = clock,
    predicate = current_screen_is(1),
    run_on = { 'CLOCK.Seconds' },
}

local weather_script = {
    action = weather,
    predicate = current_screen_is(2),
    run_on = { 'CLOCK.Seconds' },
}

register('Steelseries Apex 7 TKL', { volume_script, spotify_script, clock_script, weather_script })
