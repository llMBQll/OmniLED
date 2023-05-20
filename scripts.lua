--{"DayNames":["Monday","Tuesday","Wednesday","Thursday","Friday","Saturday","Sunday"],"MonthNames":["January","February","March","April","May","June","July","August","September","October","November","December"]}
--{"Seconds":45,"Year":2023,"MonthDay":16,"Minutes":27,"Hours":0,"WeekDay":1,"Month":4}

function clock()
    local day   = CLOCK.DayNames[CLOCK.WeekDay + 1]
    local month = CLOCK.MonthNames[CLOCK.Month + 1]

    print(string.format("%s %02d %s %04d - %02d:%02d:%02d", day, CLOCK.MonthDay, month, CLOCK.Year, CLOCK.Hours, CLOCK.Minutes, CLOCK.Seconds))
    --return {
    --    script = Text('Omegalul'),
    --    position = Position(),
    --    duration = 1000,
    --}
end

register(clock, {'CLOCK.Seconds'})