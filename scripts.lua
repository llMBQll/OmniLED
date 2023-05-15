function clock()
    print('clock updated')
    --return {
    --    script = Text('Omegalul'),
    --    position = Position(),
    --    duration = 1000,
    --}
end

register(clock, {'CLOCK.Seconds'})