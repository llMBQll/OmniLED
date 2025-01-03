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
    last_update_tick: usize,
    total_time: usize,
    current_tick: usize,
}

#[derive(Debug, PartialEq)]
pub struct Step {
    pub offset: usize,
    pub can_wrap: bool,
}

impl Animation {
    pub fn new(edge_step_time: usize, step_time: usize, steps: usize, tick: usize) -> Self {
        let total_time = match steps {
            1 => 0,
            _ => edge_step_time * 2 + (steps - 2) * step_time,
        };

        Self {
            edge_step_time,
            step_time,
            steps,
            last_update_tick: tick,
            total_time,
            current_tick: 1,
        }
    }

    pub fn step(&mut self, tick: usize) -> Step {
        let (step, can_wrap) = if self.current_tick >= self.total_time {
            (self.steps - 1, true)
        } else if self.current_tick > self.total_time - self.edge_step_time {
            (self.steps - 1, false)
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

    fn run_test(edge_time: usize, step_time: usize, steps: usize) {
        let mut tick = 0;
        let mut animation = Animation::new(edge_time, step_time, steps, 0);

        let total_time = if steps == 1 {
            0
        } else {
            2 * edge_time + step_time * (steps - 2)
        };

        assert_eq!(animation.total_time, total_time);

        for _ in 0..edge_time {
            step_and_assert_eq!(tick, animation, 0, false);
        }

        for step in 0..steps - 2 {
            for _ in 0..step_time {
                step_and_assert_eq!(tick, animation, step + 1, false);
            }
        }

        for _ in 0..edge_time - 1 {
            step_and_assert_eq!(tick, animation, steps - 1, false);
        }
        step_and_assert_eq!(tick, animation, steps - 1, true);
    }

    #[test]
    fn edge_step_time_and_step_time_equal() {
        const EDGE_TIME: usize = 6;
        const STEP_TIME: usize = 6;
        const STEPS: usize = 20;

        run_test(EDGE_TIME, STEP_TIME, STEPS);
    }

    #[test]
    fn edge_step_time_greater_than_step_time() {
        const EDGE_TIME: usize = 8;
        const STEP_TIME: usize = 2;
        const STEPS: usize = 20;

        run_test(EDGE_TIME, STEP_TIME, STEPS);
    }

    #[test]
    fn step_time_greater_than_edge_step_time() {
        const EDGE_TIME: usize = 2;
        const STEP_TIME: usize = 8;
        const STEPS: usize = 20;

        run_test(EDGE_TIME, STEP_TIME, STEPS);
    }

    #[test]
    fn single_step() {
        const EDGE_TIME: usize = 7;
        const STEP_TIME: usize = 5;
        const STEPS: usize = 1;

        let mut tick = 0;
        let mut animation = Animation::new(EDGE_TIME, STEP_TIME, STEPS, 0);

        assert_eq!(animation.total_time, 0);

        step_and_assert_eq!(tick, animation, 0, true);
    }
}
