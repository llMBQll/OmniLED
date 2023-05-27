function volume()
    return {
        data = {
            Text {
                text = AUDIO.IsMuted and 'Off' or AUDIO.Volume,
                position = Rectangle {
                    origin = Point { x = 0, y = 0 },
                    size = Size { width = SCREEN.width, height = SCREEN.height },
                },
                modifiers = Modifiers { upper = true },
            },
            Bar {
                value = AUDIO.Volume,
                position = Rectangle {
                    origin = Point { x = 0, y = 0 },
                    size = Size { width = SCREEN.width, height = SCREEN.height },
                },
                modifiers = Modifiers { vertical = true, flip_vertical = true },
            },
        },
        duration = 2000,
    }
end

function spotify()
    return {
        data = {
            Text {
                text = SPOTIFY.Artist,
                position = Rectangle {
                    origin = Point { x = 0, y = 0 },
                    size = Size { width = SCREEN.width, height = 16 },
                },
            },
            Text {
                text = SPOTIFY.Title,
                position = Rectangle {
                    origin = Point { x = 0, y = 20 },
                    size = Size { width = SCREEN.width, height = 16 },
                },
            },
            Bar {
                value = SPOTIFY.Progress * 100.0 / SPOTIFY.Duration,
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
                text = string.format("%02d%.3s", CLOCK.MonthDay, CLOCK.MonthNames[CLOCK.Month]),
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

register(volume, { 'AUDIO.Volume', 'AUDIO.IsMuted' }, { 'Steelseries Engine' })
register(spotify, { 'SPOTIFY.Progress', 'SPOTIFY.Artist', 'SPOTIFY.Title' }, { 'Steelseries Engine' })
register(clock, { 'CLOCK.Seconds' }, { 'Steelseries Engine' })