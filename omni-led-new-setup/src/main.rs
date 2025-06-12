/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2025  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use iced::border::Radius;
use iced::widget::container::Style;
use iced::widget::{column, container, radio, row, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};
use rusb::{Device, DeviceDescriptor, GlobalContext};

pub fn main() -> iced::Result {
    iced::application(Installer::title, Installer::update, Installer::view)
        .centered()
        .run()
}

#[derive(Default)]
struct InstallOptions {
    device: Option<DeviceData>,
}

pub struct Installer {
    devices: Vec<DeviceData>,
    selected_device: Option<usize>,
    install_options: InstallOptions,
}

#[derive(Debug, Clone)]
enum Message {
    DeviceSelected(usize),
}

impl Installer {
    fn title(&self) -> String {
        "OmniLED Installer".to_string()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::DeviceSelected(index) => {
                self.selected_device = Some(index);
                self.install_options.device = Some(self.devices[index].clone());
                println!("{:?}", self.devices[index]);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let title = text!("Select Device").size(30.0);
        let device_selector = scrollable(
            column(
                self.devices
                    .iter()
                    .enumerate()
                    .map(|(idx, data)| self.make_device_entry(idx, data)),
            )
            .padding(10)
            .spacing(5),
        );

        column![title, device_selector]
            .align_x(Alignment::Center)
            .into()
    }

    fn make_device_entry(&self, index: usize, data: &DeviceData) -> Element<Message> {
        let name = match (&data.vendor, &data.product) {
            (Some(vendor), Some(product)) => format!("{vendor} {product}"),
            (Some(vendor), None) => format!("{vendor}"),
            (None, Some(product)) => format!("{product}"),
            (None, None) => "Unknown".to_string(),
        };
        let name = text(name).size(20);
        let ids = text(format!("{:04X}:{:04X}", data.vendor_id, data.product_id));
        let address = text(format!("{:03} {:03}", data.bus_number, data.address));

        let desc = column![name, ids, address];

        let button = radio(
            "",
            index,
            self.selected_device.clone(),
            Message::DeviceSelected,
        )
        .width(Length::Fixed(20.0));

        container(row![button, desc].align_y(Alignment::Center))
            .padding(5)
            .width(Length::Fill)
            .style(Self::card_style)
            .into()
    }

    fn card_style(theme: &Theme) -> Style {
        let palette = theme.extended_palette();

        Style {
            background: Some(palette.background.weak.color.into()),
            border: Border {
                color: Default::default(),
                width: 2.0,
                radius: Radius::new(5.0),
            },
            ..Style::default()
        }
    }
}

impl Default for Installer {
    fn default() -> Self {
        Self {
            devices: load_devices(),
            selected_device: None,
            install_options: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct DeviceData {
    bus_number: u8,
    address: u8,
    vendor_id: u16,
    product_id: u16,
    vendor: Option<String>,
    product: Option<String>,
}

fn load_devices() -> Vec<DeviceData> {
    let mut devices: Vec<_> = rusb::devices()
        .unwrap()
        .iter()
        .map(read_device_data)
        .collect();
    devices.sort_by(|lhs, rhs| {
        lhs.bus_number
            .cmp(&rhs.bus_number)
            .then(lhs.address.cmp(&rhs.address))
    });
    devices
}

fn read_device_data(device: Device<GlobalContext>) -> DeviceData {
    let descriptor = device.device_descriptor().unwrap();
    let (vendor, product) = read_optional_device_data(&device, &descriptor);

    DeviceData {
        bus_number: device.bus_number(),
        address: device.address(),
        vendor_id: descriptor.vendor_id(),
        product_id: descriptor.product_id(),
        vendor,
        product,
    }
}

fn read_optional_device_data(
    device: &Device<GlobalContext>,
    descriptor: &DeviceDescriptor,
) -> (Option<String>, Option<String>) {
    let handle = match device.open() {
        Ok(handle) => handle,
        Err(_) => {
            return (None, None);
        }
    };

    let vendor = match handle.read_manufacturer_string_ascii(descriptor) {
        Ok(value) => Some(value),
        Err(_) => None,
    };

    let product = match handle.read_product_string_ascii(descriptor) {
        Ok(value) => Some(value),
        Err(_) => None,
    };

    (vendor, product)
}
