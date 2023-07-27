function volume()
    local first, last = string.match(AUDIO.Name, '(.+)%s%((.+)%)')
    local name = first == 'Speakers' and last or first

    return {
        data = {
            Text {
                text = AUDIO.IsMuted and 'Muted' or AUDIO.Volume,
                position = Rectangle {
                    origin = Point { x = 0, y = 0 },
                    size = Size { width = SCREEN.width, height = SCREEN.height / 2 },
                },
                modifiers = Modifiers { upper = true },
            },
            Text {
                text = name,
                position = Rectangle {
                    origin = Point { x = 0, y = SCREEN.height / 2 },
                    size = Size { width = SCREEN.width, height = SCREEN.height / 2 - 4 },
                },
                modifiers = Modifiers { scrolling = true },
            },
        },
        duration = 2000,
        auto_repeat = true,
    }
end

function spotify()
    return {
        data = {
            Bar {
                value = SPOTIFY.Progress * 100.0 / SPOTIFY.Duration,
                position = Rectangle {
                    origin = Point { x = 0, y = 0 },
                    size = Size { width = SCREEN.width, height = 2 },
                },
            },
            Text {
                text = string.format("%s - %s", SPOTIFY.Artist, SPOTIFY.Title),
                position = Rectangle {
                    origin = Point { x = 0, y = 1 },
                    size = Size { width = SCREEN.width, height = 16 },
                },
                modifiers = Modifiers { scrolling = true },
            },
            Text {
                text = string.format("%02d:%02d", CLOCK.Hours, CLOCK.Minutes),
                position = Rectangle {
                    origin = Point { x = 0, y = SCREEN.height - 16 },
                    size = Size { width = 50, height = 12 },
                },
                modifiers = Modifiers { upper = true },
            },
            Text {
                text = string.format("%.3s%02d", CLOCK.MonthNames[CLOCK.Month + 1], CLOCK.MonthDay),
                position = Rectangle {
                    origin = Point { x = SCREEN.width - 45, y = SCREEN.height - 16 },
                    size = Size { width = 45, height = 12 },
                },
                modifiers = Modifiers { upper = true },
            },
            Bar {
                value = CLOCK.Seconds * 100.0 / 59,
                position = Rectangle {
                    origin = Point { x = 0, y = SCREEN.height - 2 },
                    size = Size { width = SCREEN.width, height = 2 },
                },
            },
        },
        duration = 1000,
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
                modifiers = Modifiers { upper = true },
            },
            Text {
                text = string.format("%02d", CLOCK.Minutes),
                position = Rectangle {
                    origin = Point { x = 64, y = 0 },
                    size = Size { width = 54, height = 26 },
                },
                modifiers = Modifiers { upper = true },
            },
            Text {
                text = string.format("%.3s%02d", CLOCK.MonthNames[CLOCK.Month + 1], CLOCK.MonthDay),
                position = Rectangle {
                    origin = Point { x = 65, y = 27 },
                    size = Size { width = 54, height = 10 },
                },
                modifiers = Modifiers { upper = true },
            },
            Bar {
                value = CLOCK.Seconds * 100.0 / 59,
                position = Rectangle {
                    origin = Point { x = 0, y = SCREEN.height - 2 },
                    size = Size { width = SCREEN.width, height = 2 },
                },
            }
        },
        duration = 1000,
    }
end

register(volume, { 'AUDIO.IsMuted', 'AUDIO.Name', 'AUDIO.Volume' }, { 'Steelseries Apex 7 TKL' })
register(spotify, { 'SPOTIFY.Artist', 'SPOTIFY.Progress', 'SPOTIFY.Title' }, { 'Steelseries Apex 7 TKL' })
register(clock, { 'CLOCK.Seconds' }, { 'Steelseries Apex 7 TKL' })