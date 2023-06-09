APP_LOADER = {}

function APP_LOADER.load_app(app_config)
    if app_config.path == nil then
        LOG.warn('No path specified')
        return
    end
    if app_config.args == nil then
        LOG.warn('Starting application without parameters')
    end

    APP_LOADER.start_process(app_config)
end

function APP_LOADER.load_applications()
    f, err = loadfile(SETTINGS.applications_file, 't', { load_app = APP_LOADER.load_app, SERVER = SERVER })
    if err then
        LOG.error('Failed to load the applications file - ' .. err)
        return
    end
    f()
end
