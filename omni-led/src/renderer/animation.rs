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

#[derive(Clone)]
pub struct Animation {
    edge_step_time: usize,
    step_time: usize,
    steps: usize,
    current_tick: usize,
    last_update_tick: usize,
}

#[derive(Debug, PartialEq)]
pub struct Step {
    pub offset: usize,
    pub can_wrap: bool,
}

impl Animation {
    pub fn new(edge_step_time: usize, step_time: usize, steps: usize, tick: usize) -> Self {
        Self {
            edge_step_time,
            step_time,
            steps,
            current_tick: 1,
            last_update_tick: tick,
        }
    }

    pub fn step(&mut self, tick: usize) -> Step {
        let (step, can_wrap) =
            if self.current_tick >= self.edge_step_time * 2 + (self.steps - 1) * self.step_time {
                (self.steps, true)
            } else if self.current_tick > self.edge_step_time + (self.steps - 1) * self.step_time {
                (self.steps, false)
            } else if self.current_tick <= self.edge_step_time {
                (0, false)
            } else {
                (
                    (self.current_tick - self.edge_step_time - 1) / self.step_time + 1,
                    false,
                )
            };

        if tick != self.last_update_tick {
            self.current_tick += 1;
            self.last_update_tick = tick;
        }

        Step {
            offset: step,
            can_wrap,
        }
    }

    pub fn last_update_time(&self) -> usize {
        self.last_update_tick
    }

    pub fn reset(&mut self) {
        self.current_tick = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! step_and_assert_eq {
        ($tick:ident, $anim:ident, $step:expr, $can_wrap:expr) => {
            $tick += 1;
            assert_eq!(
                $anim.step($tick),
                Step {
                    offset: $step,
                    can_wrap: $can_wrap
                }
            );
        };
    }

    #[test]
    fn edge_step_time_and_step_time_equal() {
        let mut tick = 0;
        let mut animation = Animation::new(3, 3, 3, 0);

        step_and_assert_eq!(tick, animation, 0, false);
        step_and_assert_eq!(tick, animation, 0, false);
        step_and_assert_eq!(tick, animation, 0, false);

        step_and_assert_eq!(tick, animation, 1, false);
        step_and_assert_eq!(tick, animation, 1, false);
        step_and_assert_eq!(tick, animation, 1, false);

        step_and_assert_eq!(tick, animation, 2, false);
        step_and_assert_eq!(tick, animation, 2, false);
        step_and_assert_eq!(tick, animation, 2, false);

        step_and_assert_eq!(tick, animation, 3, false);
        step_and_assert_eq!(tick, animation, 3, false);
        step_and_assert_eq!(tick, animation, 3, true);
    }

    #[test]
    fn edge_step_time_greater_than_step_time() {
        let mut tick = 0;
        let mut animation = Animation::new(4, 3, 2, 0);

        step_and_assert_eq!(tick, animation, 0, false);
        step_and_assert_eq!(tick, animation, 0, false);
        step_and_assert_eq!(tick, animation, 0, false);
        step_and_assert_eq!(tick, animation, 0, false);

        step_and_assert_eq!(tick, animation, 1, false);
        step_and_assert_eq!(tick, animation, 1, false);
        step_and_assert_eq!(tick, animation, 1, false);

        step_and_assert_eq!(tick, animation, 2, false);
        step_and_assert_eq!(tick, animation, 2, false);
        step_and_assert_eq!(tick, animation, 2, false);
        step_and_assert_eq!(tick, animation, 2, true);
    }

    #[test]
    fn step_time_greater_than_edge_step_time() {
        let mut tick = 0;
        let mut animation = Animation::new(2, 3, 4, 0);

        step_and_assert_eq!(tick, animation, 0, false);
        step_and_assert_eq!(tick, animation, 0, false);

        step_and_assert_eq!(tick, animation, 1, false);
        step_and_assert_eq!(tick, animation, 1, false);
        step_and_assert_eq!(tick, animation, 1, false);

        step_and_assert_eq!(tick, animation, 2, false);
        step_and_assert_eq!(tick, animation, 2, false);
        step_and_assert_eq!(tick, animation, 2, false);

        step_and_assert_eq!(tick, animation, 3, false);
        step_and_assert_eq!(tick, animation, 3, false);
        step_and_assert_eq!(tick, animation, 3, false);

        step_and_assert_eq!(tick, animation, 4, false);
        step_and_assert_eq!(tick, animation, 4, true);
    }
}
