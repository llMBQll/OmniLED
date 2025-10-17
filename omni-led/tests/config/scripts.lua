EVENTS:register('*', function(event, value)
    if event == 'OMNILED.Update' and value == 10 then
        end_test()
    end
end)