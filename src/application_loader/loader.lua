LOADER = {}

function LOADER.load_app(app_config)
    if app_config.path == nil then
        -- TODO log proper error
        print('\tERROR: null path')
        return
    end
    if app_config.args == nil then
        -- TODO log proper warning: running application without parameters
        print('\tWARN: no args')
        return
    end

    LOADER.start_application(app_config)
end

function LOADER.load_applications()
    f, err = loadfile(SETTINGS.applications_file, 't', { load_app = LOADER.load_app })
    if err then
        -- TODO log error
        return
    end
    f()
end
