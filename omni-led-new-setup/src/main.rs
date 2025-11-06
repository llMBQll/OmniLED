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

mod devices;
mod logging;

use crate::devices::load_supported_devices;
use crate::logging::LogHandleImpl;
use iced::border::Radius;
use iced::widget::container::Style;
use iced::widget::{
    button, column, container, horizontal_space, radio, row, scrollable, text, toggler,
};
use iced::{Alignment, Border, Element, Font, Length, Theme};
use mlua::Lua;
use omni_led_lib::common::common::load_internal_functions;
use omni_led_lib::common::user_data::UserDataRef;
use omni_led_lib::constants::configs::{ConfigType, Configs};
use omni_led_lib::constants::constants::Constants;
use omni_led_lib::devices::devices::Devices;
use omni_led_lib::logging::logger::Log;
use rusb::{Device, DeviceDescriptor, GlobalContext};

const DEVICES: &str = include_str!("../../config/devices.lua");

pub fn main() -> iced::Result {
    logging::init();

    let lua = init_lua();
    load_supported_devices(&lua);

    iced::application(Installer::title, Installer::update, Installer::view)
        .font(include_bytes!(
            "../../assets/fonts/FiraMono/FiraMono-Bold.ttf"
        ))
        .default_font(Font::with_name("FiraMono"))
        .centered()
        .run()
}

fn init_lua() -> Lua {
    let lua = Lua::new();
    load_internal_functions(&lua);

    Constants::load(&lua, None);

    Log::load(&lua, LogHandleImpl);

    // Config directory doesn't exist yet, override device config from memory
    UserDataRef::<Configs>::load(&lua)
        .get_mut()
        .store_config(ConfigType::Devices, DEVICES)
        .unwrap();
    Devices::load(&lua);

    lua
}

#[derive(Default, Debug)]
struct InstallOptions {
    device: Option<DeviceData>,
    enable_autostart: bool,
    update_udev_rules: bool,
}

pub struct Installer {
    devices: Vec<DeviceData>,
    selected_device: Option<usize>,
    screen: Screen,
    install_options: InstallOptions,
}

#[derive(Debug, Clone)]
enum Message {
    Back,
    Next,
    DeviceSelected(usize),
    AutostartToggle(bool),
    UdevToggle(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Welcome,
    DeviceSelect,
    Settings,
    Finish,
}

impl Screen {
    const ALL: &'static [Self] = &[
        Self::Welcome,
        Self::DeviceSelect,
        Self::Settings,
        Self::Finish,
    ];

    pub fn next(self) -> Option<Screen> {
        Self::ALL
            .get(
                Self::ALL
                    .iter()
                    .copied()
                    .position(|screen| screen == self)
                    .expect("Screen must exist")
                    + 1,
            )
            .copied()
    }

    pub fn back(self) -> Option<Screen> {
        let position = Self::ALL
            .iter()
            .copied()
            .position(|screen| screen == self)
            .expect("Screen must exist");

        if position > 0 {
            Some(Self::ALL[position - 1])
        } else {
            None
        }
    }
}

impl Installer {
    fn title(&self) -> String {
        "OmniLED Installer".to_string()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Back => {
                self.screen = self.screen.back().unwrap_or(self.screen);
            }
            Message::Next => {
                self.screen = self.screen.next().unwrap_or(self.screen);
                if self.screen == Screen::Finish {
                    println!("{:#?}", self.install_options);
                }
            }
            Message::DeviceSelected(index) => {
                self.selected_device = Some(index);
                self.install_options.device = Some(self.devices[index].clone());
                println!("{:?}", self.devices[index]);
            }
            Message::AutostartToggle(value) => {
                self.install_options.enable_autostart = value;
            }
            Message::UdevToggle(value) => {
                self.install_options.update_udev_rules = value;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let screen: Element<Message> = match self.screen {
            Screen::Welcome => self.welcome(),
            Screen::DeviceSelect => self.device_select(),
            Screen::Settings => self.settings(),
            Screen::Finish => self.finish(),
        };
        let screen = container(scrollable(screen))
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        let back = button("Back").on_press(Message::Back);
        let next = button("Next").on_press(Message::Next);
        let buttons = row![]
            .push_maybe(self.screen.back().map(|_| back))
            .push(horizontal_space())
            .push_maybe(self.can_advance().then_some(next))
            .padding(20);

        column![screen, buttons]
            .align_x(Alignment::Center)
            .spacing(10)
            .into()
    }

    fn can_advance(&self) -> bool {
        match self.screen {
            Screen::Welcome => true,
            Screen::DeviceSelect => self.selected_device.is_some(),
            Screen::Settings => true,
            Screen::Finish => false,
        }
    }

    fn welcome(&self) -> Element<Message> {
        let title = text!("OmniLED Installer").size(30.0);

        title.into()
    }

    fn device_select(&self) -> Element<Message> {
        let title = text!("Select Device").size(30.0);
        let device_selector = column(
            self.devices
                .iter()
                .enumerate()
                .map(|(idx, data)| self.make_device_entry(idx, data)),
        )
        .padding(10)
        .spacing(5);

        column![title, device_selector]
            .align_x(Alignment::Center)
            .into()
    }

    fn settings(&self) -> Element<Message> {
        let content = column![].align_x(Alignment::Center);

        let content = content.push(text!("Settings").size(30.0));

        let content = content.push(Self::make_card(
            row![
                toggler(self.install_options.enable_autostart).on_toggle(Message::AutostartToggle),
                column![
                    text("Auto Start").size(20),
                    text("Start OmniLED on computer start-up").size(12)
                ]
            ]
            .align_y(Alignment::Center)
            .into(),
        ));

        let content = content.push_maybe(
            // TODO allow changing filename
            cfg!(target_os = "linux").then_some(Self::make_card(
                row![
                    toggler(self.install_options.update_udev_rules).on_toggle(Message::UdevToggle),
                    column![
                        text("Allow USB Access").size(20),
                        text("Add udev rule to allow OmniLED access USB devices").size(12)
                    ]
                ]
                .align_y(Alignment::Center)
                .into(),
            )),
        );

        content.into()
    }

    fn finish(&self) -> Element<Message> {
        let title = text!("Install Finished").size(30.0);

        title.into()
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

        Self::make_card(row![button, desc].align_y(Alignment::Center).into())
    }

    fn make_card(element: Element<Message>) -> Element<Message> {
        container(element)
            .padding(10)
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
                radius: Radius::new(10.0),
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
            screen: Screen::Welcome,
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
