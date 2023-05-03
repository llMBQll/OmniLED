LOADER = {}

function LOADER.load_app(app_config)
    if app_config.path == nil then
        LOG.warn('No path specified')
        return
    end
    if app_config.args == nil then
        LOG.warn('Starting application without parameters')
    end

    LOADER.start_application(app_config)
end

function LOADER.load_applications()
    f, err = loadfile(SETTINGS.applications_file, 't', { load_app = LOADER.load_app })
    if err then
        LOG.error('Failed to load the applications file - ' .. err)
        return
    end
    f()
end
