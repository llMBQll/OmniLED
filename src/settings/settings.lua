local _user_input = {}

f, err = loadfile('settings.lua', 't', _user_input)
if err then
    LOG.error(err)
    return
end
f()

for key, val in pairs(_user_input) do
    if type(key) ~= 'string' then
        LOG.warn('Non-string key detected - "' .. tostring(key) .. '"')
    elseif not SETTINGS.key_exists(key) then
        LOG.warn('Unrecognized setting "' .. key .. '"')
    else
        SETTINGS[key] = val
    end
end

_user_input = nil