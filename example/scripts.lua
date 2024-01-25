function volume()
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
        repeat_once = true,
    }
end

local SPOTIFY_DURATION = PLATFORM.Os == 'windows' and 5000 or 1000
function spotify()
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
        repeat_to_fit = true,
    }
end

function clock()
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

function weather()
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
        repeat_once = true,
    }
end

register(volume, { '1(AUDIO.IsMuted)', '1(AUDIO.Name)', '1(AUDIO.Volume)' }, { 'Steelseries Apex 7 TKL' })
register(spotify, { '1(SPOTIFY.Artist)', '1(SPOTIFY.Progress)', '1(SPOTIFY.Title)' }, { 'Steelseries Apex 7 TKL' })
register(clock, { '1(CLOCK.Seconds)' }, { 'Steelseries Apex 7 TKL' })
register(weather, { '2(CLOCK.Seconds)' }, { 'Steelseries Apex 7 TKL' })

local current = '1'

EVENTS:set_filter(function (event, data)
    local res = KEY_COMBINATION_HANDLER:handle_key(event, data)
    if res == 'Toggle' then
        if current == '1' then
            current = '2'
        else
            current = '1'
        end
        EVENTS:reset_state()
    end

    return EVENTS:make_prefixed(current, event)
end)

KEY_COMBINATION_HANDLER:register_combination('Toggle', {'KEY(RAlt)', 'KEY(Slash)'})