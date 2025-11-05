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

use crate::renderer::animation::{Animation, State};
use crate::script_handler::script_data_types::Repeat;

#[derive(Clone)]
pub struct AnimationGroup {
    items: Vec<Item>,
    new_data: bool,
    keep_in_sync: bool,
}

#[derive(Clone)]
struct Item {
    hash: u64,
    animation: Animation,
    accessed: bool,
}

impl AnimationGroup {
    pub fn new(keep_in_sync: bool) -> Self {
        Self {
            items: Vec::new(),
            new_data: false,
            keep_in_sync,
        }
    }

    pub fn entry(&mut self, hash: u64) -> Entry<'_> {
        let mut index = None;
        for (i, item) in self.items.iter_mut().enumerate() {
            if item.hash == hash {
                index = Some(i);
                break;
            }
        }

        match index {
            Some(index) => Entry::Occupied(OccupiedEntry {
                _hash: hash,
                item: &mut self.items[index],
            }),
            None => Entry::Vacant(VacantEntry { hash, group: self }),
        }
    }

    pub fn reset(&mut self) {
        for item in &mut self.items {
            item.animation.reset();
        }
    }

    pub fn pre_sync(&mut self) {
        self.items.retain_mut(|item| {
            if self.new_data && self.keep_in_sync {
                item.animation.reset();
            }
            std::mem::replace(&mut item.accessed, false)
        });
        self.new_data = false;
    }

    pub fn sync(&mut self) {
        if self.keep_in_sync {
            let all_can_wrap = self.items.iter().all(|item| item.animation.can_wrap());
            if all_can_wrap {
                for item in &mut self.items {
                    if item.animation.repeat_type() != Repeat::Once {
                        item.animation.reset();
                    }
                }
            }
        } else {
            for item in &mut self.items {
                if item.animation.can_wrap() && item.animation.repeat_type() != Repeat::Once {
                    item.animation.reset();
                }
            }
        }
    }

    pub fn states(&self) -> Vec<State> {
        self.items
            .iter()
            .map(|item| item.animation.state())
            .collect()
    }
}

pub enum Entry<'a> {
    Occupied(OccupiedEntry<'a>),
    Vacant(VacantEntry<'a>),
}

impl<'a> Entry<'a> {
    pub fn or_insert_with<F: FnOnce() -> Animation>(self, f: F) -> &'a mut Animation {
        match self {
            Entry::Occupied(entry) => {
                entry.item.accessed = true;
                &mut entry.item.animation
            }
            Entry::Vacant(entry) => {
                entry.group.new_data = true;
                entry.group.items.push(Item {
                    hash: entry.hash,
                    animation: f(),
                    accessed: true,
                });
                let index = entry.group.items.len() - 1;
                &mut entry.group.items[index].animation
            }
        }
    }

    pub fn unwrap(self) -> &'a mut Animation {
        match self {
            Entry::Occupied(entry) => {
                entry.item.accessed = true;
                &mut entry.item.animation
            }
            Entry::Vacant(entry) => {
                panic!("Entry with hash {} doesn't exist", entry.hash);
            }
        }
    }
}

pub struct OccupiedEntry<'a> {
    _hash: u64,
    item: &'a mut Item,
}

pub struct VacantEntry<'a> {
    hash: u64,
    group: &'a mut AnimationGroup,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::animation::Animation;
    use crate::script_handler::script_data_types::Repeat;
    use test_case::test_case;

    fn run_animation(group: &mut AnimationGroup, n: usize) -> Vec<Vec<(usize, State)>> {
        let mut data = Vec::new();
        for _ in 0..n {
            let mut current_step_data = Vec::new();
            group.pre_sync();
            for item in &mut group.items {
                let step = item.animation.step();
                let state = item.animation.state();
                current_step_data.push((step, state));

                item.accessed = true;
            }
            group.sync();
            data.push(current_step_data);
        }
        data
    }

    fn create_test_animation() -> Animation {
        Animation::new(1, 2, 3, Repeat::Once)
    }

    #[test_case(true  ; "Synced")]
    #[test_case(false ; "Not synced")]
    fn insertion(synced: bool) {
        const HASH: u64 = 1337;

        let mut group = AnimationGroup::new(synced);
        let mut insert_count = 0;

        _ = group.entry(HASH).or_insert_with(|| {
            insert_count += 1;
            create_test_animation()
        });

        assert_eq!(insert_count, 1);
        assert_eq!(group.items.len(), 1);
        assert_eq!(group.items[0].hash, HASH);
        assert_eq!(group.items[0].accessed, true);
        assert_eq!(group.new_data, true);

        group.items[0].accessed = false;
        group.new_data = false;

        _ = group.entry(HASH).or_insert_with(|| {
            insert_count += 1;
            create_test_animation()
        });

        assert_eq!(insert_count, 1);
        assert_eq!(group.items.len(), 1);
        assert_eq!(group.items[0].hash, HASH);
        assert_eq!(group.items[0].accessed, true);
        assert_eq!(group.new_data, false);
    }

    #[test_case(true  ; "Synced")]
    #[test_case(false ; "Not synced")]
    fn pre_sync_resets_accessed_flag(synced: bool) {
        let mut group = AnimationGroup::new(synced);
        group.entry(1).or_insert_with(|| create_test_animation());
        group.entry(2).or_insert_with(|| create_test_animation());
        group.entry(3).or_insert_with(|| create_test_animation());

        assert_eq!(group.items.len(), 3);
        for item in &group.items {
            assert_eq!(item.accessed, true);
        }

        group.pre_sync();

        assert_eq!(group.items.len(), 3);
        for item in &group.items {
            assert_eq!(item.accessed, false);
        }
    }

    #[test_case(true  ; "Synced")]
    #[test_case(false ; "Not synced")]
    fn pre_sync_removes_unaccessed_items(synced: bool) {
        let mut group = AnimationGroup::new(synced);
        group.entry(1).or_insert_with(|| create_test_animation());
        group.entry(2).or_insert_with(|| create_test_animation());
        group.entry(3).or_insert_with(|| create_test_animation());

        assert_eq!(group.items.len(), 3);

        group.items[0].accessed = true;
        group.items[1].accessed = false;
        group.items[2].accessed = true;

        group.pre_sync();

        assert_eq!(group.items.len(), 2);
        assert_eq!(group.items[0].hash, 1);
        assert_eq!(group.items[1].hash, 3);
    }

    #[test_case(true  ; "Synced")]
    #[test_case(false ; "Not synced")]
    fn animation_run_equal_repeat_once(synced: bool) {
        let animation = Animation::new(1, 1, 2, Repeat::Once);

        let mut group = AnimationGroup::new(synced);
        group.entry(1).or_insert_with(|| animation.clone());
        group.entry(2).or_insert_with(|| animation.clone());

        let r = run_animation(&mut group, 3);
        assert_eq!(r[0], vec![(0, State::InProgress), (0, State::InProgress)]);
        assert_eq!(r[1], vec![(1, State::Finished), (1, State::Finished)]);
        assert_eq!(r[2], vec![(1, State::Finished), (1, State::Finished)]);
    }

    #[test_case(true  ; "Synced")]
    #[test_case(false ; "Not synced")]
    fn animation_run_different_repeat_once(synced: bool) {
        let animation_a = Animation::new(1, 1, 2, Repeat::Once);
        let animation_b = Animation::new(1, 1, 4, Repeat::Once);

        let mut group = AnimationGroup::new(synced);
        group.entry(1).or_insert_with(|| animation_a);
        group.entry(2).or_insert_with(|| animation_b);

        let r = run_animation(&mut group, 5);
        assert_eq!(r[0], vec![(0, State::InProgress), (0, State::InProgress)]);
        assert_eq!(r[1], vec![(1, State::Finished), (1, State::InProgress)]);
        assert_eq!(r[2], vec![(1, State::Finished), (2, State::InProgress)]);
        assert_eq!(r[3], vec![(1, State::Finished), (3, State::Finished)]);
        assert_eq!(r[4], vec![(1, State::Finished), (3, State::Finished)]);
    }

    #[test_case(true  ; "Synced")]
    #[test_case(false ; "Not synced")]
    fn animation_run_equal_repeat_for_duration(synced: bool) {
        let animation = Animation::new(1, 1, 2, Repeat::ForDuration);

        let mut group = AnimationGroup::new(synced);
        group.entry(1).or_insert_with(|| animation.clone());
        group.entry(2).or_insert_with(|| animation.clone());

        let r = run_animation(&mut group, 4);
        assert_eq!(r[0], vec![(0, State::CanFinish), (0, State::CanFinish)]);
        assert_eq!(r[1], vec![(1, State::CanFinish), (1, State::CanFinish)]);
        assert_eq!(r[2], vec![(0, State::CanFinish), (0, State::CanFinish)]);
        assert_eq!(r[3], vec![(1, State::CanFinish), (1, State::CanFinish)]);
    }

    #[test_case(true  ; "Synced")]
    #[test_case(false ; "Not synced")]
    fn animation_run_different_repeat_for_duration(synced: bool) {
        let animation_a = Animation::new(1, 1, 2, Repeat::ForDuration);
        let animation_b = Animation::new(1, 1, 4, Repeat::ForDuration);

        let mut group = AnimationGroup::new(synced);
        group.entry(1).or_insert_with(|| animation_a);
        group.entry(2).or_insert_with(|| animation_b);

        let r = run_animation(&mut group, 6);
        if synced {
            assert_eq!(r[0], vec![(0, State::CanFinish), (0, State::CanFinish)]);
            assert_eq!(r[1], vec![(1, State::CanFinish), (1, State::CanFinish)]);
            assert_eq!(r[2], vec![(1, State::CanFinish), (2, State::CanFinish)]);
            assert_eq!(r[3], vec![(1, State::CanFinish), (3, State::CanFinish)]);
            assert_eq!(r[4], vec![(0, State::CanFinish), (0, State::CanFinish)]);
            assert_eq!(r[5], vec![(1, State::CanFinish), (1, State::CanFinish)]);
        } else {
            assert_eq!(r[0], vec![(0, State::CanFinish), (0, State::CanFinish)]);
            assert_eq!(r[1], vec![(1, State::CanFinish), (1, State::CanFinish)]);
            assert_eq!(r[2], vec![(0, State::CanFinish), (2, State::CanFinish)]);
            assert_eq!(r[3], vec![(1, State::CanFinish), (3, State::CanFinish)]);
            assert_eq!(r[4], vec![(0, State::CanFinish), (0, State::CanFinish)]);
            assert_eq!(r[5], vec![(1, State::CanFinish), (1, State::CanFinish)]);
        }
    }

    #[test_case(true  ; "Synced")]
    #[test_case(false ; "Not synced")]
    fn animation_run_different_mixed(synced: bool) {
        let animation_a = Animation::new(1, 1, 2, Repeat::ForDuration);
        let animation_b = Animation::new(1, 1, 4, Repeat::Once);

        let mut group = AnimationGroup::new(synced);
        group.entry(1).or_insert_with(|| animation_a);
        group.entry(2).or_insert_with(|| animation_b);

        let r = run_animation(&mut group, 5);
        if synced {
            assert_eq!(r[0], vec![(0, State::CanFinish), (0, State::InProgress)]);
            assert_eq!(r[1], vec![(1, State::CanFinish), (1, State::InProgress)]);
            assert_eq!(r[2], vec![(1, State::CanFinish), (2, State::InProgress)]);
            assert_eq!(r[3], vec![(1, State::CanFinish), (3, State::Finished)]);
            assert_eq!(r[4], vec![(0, State::CanFinish), (3, State::Finished)]);
        } else {
            assert_eq!(r[0], vec![(0, State::CanFinish), (0, State::InProgress)]);
            assert_eq!(r[1], vec![(1, State::CanFinish), (1, State::InProgress)]);
            assert_eq!(r[2], vec![(0, State::CanFinish), (2, State::InProgress)]);
            assert_eq!(r[3], vec![(1, State::CanFinish), (3, State::Finished)]);
            assert_eq!(r[4], vec![(0, State::CanFinish), (3, State::Finished)]);
        }
    }

    #[test_case(true  ; "Synced")]
    #[test_case(false ; "Not synced")]
    fn animation_run_new_data_repeat_for_duration(synced: bool) {
        let animation_a = Animation::new(1, 1, 2, Repeat::ForDuration);
        let animation_b = Animation::new(1, 1, 4, Repeat::ForDuration);
        let animation_c = Animation::new(1, 1, 4, Repeat::ForDuration);

        let mut group = AnimationGroup::new(synced);
        group.entry(1).or_insert_with(|| animation_a);
        group.entry(2).or_insert_with(|| animation_b);

        let r = run_animation(&mut group, 3);
        if synced {
            assert_eq!(r[0], vec![(0, State::CanFinish), (0, State::CanFinish)]);
            assert_eq!(r[1], vec![(1, State::CanFinish), (1, State::CanFinish)]);
            assert_eq!(r[2], vec![(1, State::CanFinish), (2, State::CanFinish)]);
        } else {
            assert_eq!(r[0], vec![(0, State::CanFinish), (0, State::CanFinish)]);
            assert_eq!(r[1], vec![(1, State::CanFinish), (1, State::CanFinish)]);
            assert_eq!(r[2], vec![(0, State::CanFinish), (2, State::CanFinish)]);
        }

        // Simulate new data replacing one of the entries
        group.entry(3).or_insert_with(|| animation_c);
        group.items[1].accessed = false;

        let r = run_animation(&mut group, 3);
        if synced {
            assert_eq!(r[0], vec![(0, State::CanFinish), (0, State::CanFinish)]);
            assert_eq!(r[1], vec![(1, State::CanFinish), (1, State::CanFinish)]);
            assert_eq!(r[2], vec![(1, State::CanFinish), (2, State::CanFinish)]);
        } else {
            assert_eq!(r[0], vec![(1, State::CanFinish), (0, State::CanFinish)]);
            assert_eq!(r[1], vec![(0, State::CanFinish), (1, State::CanFinish)]);
            assert_eq!(r[2], vec![(1, State::CanFinish), (2, State::CanFinish)]);
        }
    }

    #[test_case(true  ; "Synced")]
    #[test_case(false ; "Not synced")]
    fn animation_run_new_data_repeat_once(synced: bool) {
        let animation_a = Animation::new(1, 1, 2, Repeat::Once);
        let animation_b = Animation::new(1, 1, 4, Repeat::Once);
        let animation_c = Animation::new(1, 1, 4, Repeat::Once);

        let mut group = AnimationGroup::new(synced);
        group.entry(1).or_insert_with(|| animation_a);
        group.entry(2).or_insert_with(|| animation_b);

        let r = run_animation(&mut group, 3);
        if synced {
            assert_eq!(r[0], vec![(0, State::InProgress), (0, State::InProgress)]);
            assert_eq!(r[1], vec![(1, State::Finished), (1, State::InProgress)]);
            assert_eq!(r[2], vec![(1, State::Finished), (2, State::InProgress)]);
        } else {
            assert_eq!(r[0], vec![(0, State::InProgress), (0, State::InProgress)]);
            assert_eq!(r[1], vec![(1, State::Finished), (1, State::InProgress)]);
            assert_eq!(r[2], vec![(1, State::Finished), (2, State::InProgress)]);
        }

        // Simulate new data replacing one of the entries
        group.entry(3).or_insert_with(|| animation_c);
        group.items[1].accessed = false;

        let r = run_animation(&mut group, 3);
        if synced {
            assert_eq!(r[0], vec![(0, State::InProgress), (0, State::InProgress)]);
            assert_eq!(r[1], vec![(1, State::Finished), (1, State::InProgress)]);
            assert_eq!(r[2], vec![(1, State::Finished), (2, State::InProgress)]);
        } else {
            assert_eq!(r[0], vec![(1, State::Finished), (0, State::InProgress)]);
            assert_eq!(r[1], vec![(1, State::Finished), (1, State::InProgress)]);
            assert_eq!(r[2], vec![(1, State::Finished), (2, State::InProgress)]);
        }
    }
}
