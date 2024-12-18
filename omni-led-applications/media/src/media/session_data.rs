/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
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

use omni_led_derive::IntoProto;
use std::time::Duration;

#[derive(Clone, IntoProto)]
#[proto(rename_all(PascalCase))]
pub struct SessionData {
    pub artist: String,
    pub title: String,
    #[proto(transform(Self::duration_into_ms))]
    pub progress: Duration,
    #[proto(transform(Self::duration_into_ms))]
    pub duration: Duration,
    pub playing: bool,
}

impl SessionData {
    fn duration_into_ms(duration: Duration) -> i64 {
        duration.as_millis() as i64
    }
}
