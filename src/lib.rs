use std::sync::{
    Mutex,
    MutexGuard,
};

use bumpalo::Bump;

#[derive(Default)]
pub struct Herd {
    allocs: Vec<Mutex<Bump>>,
}

impl Herd {
    pub fn new(max_instances: usize) -> Herd {
        let mut allocs = Vec::<Mutex<Bump>>::new();
        allocs.resize_with(max_instances, || Mutex::new(Default::default()));
        Herd{allocs}
    }

    pub fn reset(&mut self) {
        for e in &mut self.allocs {
            e.get_mut().unwrap().reset();
        }
    }

    pub fn get<'h>(&'h self) -> Member<'h> {
        for alloc in &self.allocs {
            if let Ok(lock) = alloc.try_lock() {
                return Member {
                    arena: lock,
                };
            }
        }
        panic!("all {} allocators locked", self.allocs.len());
    }
}

pub struct Member<'h> {
    /// !Send, unfortunately.
    arena: MutexGuard<'h, Bump>,
}

impl<'h> Member<'h> {
    pub fn alloc<T>(&self, val: T) -> &'h T {
        // self.arena lasts for 'h.
        // why can't we return &'h self.arena[]?
        // because &mut self.arena could invalidate references.
        // but it's impossible to access &mut self.arena.
        let result = self.arena.alloc(val) as *const _;
        unsafe { &*result }
    }
}
