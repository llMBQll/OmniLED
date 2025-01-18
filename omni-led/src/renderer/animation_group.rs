use crate::renderer::animation::{Animation, State};

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

    pub fn entry(&mut self, hash: u64) -> Entry {
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

    pub fn pre_sync(&mut self) {
        self.items.retain_mut(|item| {
            if item.accessed {
                if self.new_data && self.keep_in_sync {
                    item.animation.reset();
                    item.accessed = false;
                }
                true
            } else {
                false
            }
        });
        self.new_data = false;
    }

    pub fn sync(&mut self) {
        if self.keep_in_sync {
            let all_can_wrap = self.items.iter().all(|item| item.animation.can_wrap());
            if all_can_wrap {
                for item in &mut self.items {
                    item.animation.reset();
                }
            }
        } else {
            for item in &mut self.items {
                if item.animation.can_wrap() {
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
