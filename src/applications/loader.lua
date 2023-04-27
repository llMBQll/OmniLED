LOADER = {}

function LOADER.load_app(data)
    if data.path == nil then
        -- TODO log error
        return
    end
    if data.args == nil or #data.args == 0 then
        -- TODO log warning: running application without parameters
        return
    end
end

function LOADER.load_applications()
    f, err = loadfile(SETTINGS.applications_filename, 't', { load_app = LOADER.load_app })
    if err then
        -- TODO log error
        return
    end
    f()
end
