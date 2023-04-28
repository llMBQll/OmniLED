LOADER = {}

function LOADER.start_application(app_config)
--    Dummy function, actual implementation is set from the Rust side
end

function LOADER.load_app(app_config)
    if app_config.path == nil then
        -- TODO log error
        return
    end
    if app_config.args == nil or #app_config.args == 0 then
        -- TODO log warning: running application without parameters
        return
    end
    LOADER.start_application(app_config)
end

function LOADER.load_applications()
    f, err = loadfile(SETTINGS.applications_filename, 't', { load_app = LOADER.load_app })
    if err then
        -- TODO log error
        return
    end
    f()
end
