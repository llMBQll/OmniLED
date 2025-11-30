usb_device {
    name = 'SteelSeries Apex 5',
    screen_size = {
        width = 128,
        height = 40,
    },
    usb_settings = {
        vendor_id = '0x1038',
        product_id = '0x161C',
        interface = '0x01',
        alternate_setting = '0x00',
        request_type = '0x21',
        request = '0x09',
        value = '0x0300',
        index = '0x01',
    },
    transform = transform_data({ prepend = {0x61}, append = {0x00} }),
    memory_layout = 'SteelSeries',
}

usb_device {
    name = 'SteelSeries Apex 7 TKL',
    screen_size = {
        width = 128,
        height = 40,
    },
    usb_settings = {
        vendor_id = '0x1038',
        product_id = '0x1618',
        interface = '0x01',
        alternate_setting = '0x00',
        request_type = '0x21',
        request = '0x09',
        value = '0x0300',
        index = '0x01',
    },
    transform = transform_data({ prepend = {0x61}, append = {0x00} }),
    memory_layout = 'SteelSeries',
}

usb_device {
    name = 'SteelSeries Apex Pro',
    screen_size = {
        width = 128,
        height = 40,
    },
    usb_settings = {
        vendor_id = '0x1038',
        product_id = '0x1610',
        interface = '0x01',
        alternate_setting = '0x00',
        request_type = '0x21',
        request = '0x09',
        value = '0x0300',
        index = '0x01',
    },
    transform = transform_data({ prepend = {0x61}, append = {0x00} }),
    memory_layout = 'SteelSeries',
}

usb_device {
    name = 'SteelSeries Apex Pro TKL Wireless Gen 3',
    screen_size = {
        width = 128,
        height = 40,
    },
    usb_settings = {
        vendor_id = '0x1038',
        product_id = '0x1646',
        interface = '0x03',
        alternate_setting = '0x00',
        request_type = '0x21',
        request = '0x09',
        value = '0x0300',
        index = '0x03',
    },
    transform = transform_data({ prepend = {0x0a} }),
    memory_layout = 'SteelSeries2',
}

emulator {
    name = 'Emulator',
    screen_size = {
        width = 128,
        height = 40,
    },
}
