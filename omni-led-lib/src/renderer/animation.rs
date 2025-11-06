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

use crate::script_handler::script_data_types::Repeat;

#[derive(Clone, Debug)]
pub struct Animation {
    edge_step_time: usize,
    step_time: usize,
    steps: usize,
    total_time: usize,
    repeat: Repeat,
    current_tick: usize,
    can_wrap: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    InProgress,
    Finished,
    CanFinish,
}

impl Animation {
    pub fn new(edge_step_time: usize, step_time: usize, steps: usize, repeat: Repeat) -> Self {
        let total_time = match steps {
            1 => 0,
            _ => edge_step_time * 2 + (steps - 2) * step_time,
        };

        Self {
            edge_step_time,
            step_time,
            steps,
            total_time,
            repeat,
            current_tick: 1,
            can_wrap: false,
        }
    }

    pub fn step(&mut self) -> usize {
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

        self.current_tick += 1;
        self.can_wrap = can_wrap;

        step
    }

    pub fn state(&self) -> State {
        match (self.repeat, self.can_wrap) {
            (Repeat::Once, false) => State::InProgress,
            (Repeat::Once, true) => State::Finished,
            (Repeat::ForDuration, _) => State::CanFinish,
        }
    }

    pub fn repeat_type(&self) -> Repeat {
        self.repeat
    }

    pub fn can_wrap(&self) -> bool {
        self.can_wrap
    }

    pub fn reset(&mut self) {
        self.current_tick = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! step_and_assert_eq {
        ($anim:ident, $step:expr, $can_wrap:expr, $state:expr) => {
            assert_eq!($anim.step(), $step);
            assert_eq!($anim.can_wrap(), $can_wrap);
            assert_eq!($anim.state(), $state);
        };
    }

    fn run_test(edge_time: usize, step_time: usize, steps: usize) {
        let mut animation = Animation::new(edge_time, step_time, steps, Repeat::Once);

        let total_time = if steps == 1 {
            0
        } else {
            2 * edge_time + step_time * (steps - 2)
        };

        assert_eq!(animation.total_time, total_time);

        for _ in 0..edge_time {
            step_and_assert_eq!(animation, 0, false, State::InProgress);
        }

        for step in 0..steps - 2 {
            for _ in 0..step_time {
                step_and_assert_eq!(animation, step + 1, false, State::InProgress);
            }
        }

        for _ in 0..edge_time - 1 {
            step_and_assert_eq!(animation, steps - 1, false, State::InProgress);
        }
        step_and_assert_eq!(animation, steps - 1, true, State::Finished);
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

        let mut animation = Animation::new(EDGE_TIME, STEP_TIME, STEPS, Repeat::Once);

        assert_eq!(animation.total_time, 0);

        step_and_assert_eq!(animation, 0, true, State::Finished);
    }
}
