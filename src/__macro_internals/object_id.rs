use parking_lot::RwLock;
use std::{any::type_name, sync::Arc};

const ENFORCED_MAX: usize = 0x7FFFFFFF;

enum FreeListNode<T> {
    Data(T),
    Free(usize),
}

struct IdManagerData<T: Send + Sync> {
    nodes: Vec<FreeListNode<Arc<T>>>,
    head: usize,
}
impl<T: Send + Sync> IdManagerData<T> {
    const fn new() -> Self {
        IdManagerData {
            nodes: Vec::new(),
            head: 0,
        }
    }

    fn get(&self, id: u32) -> Arc<T> {
        let id = id as usize;
        if id >= self.nodes.len() {
            panic!(
                "freelist for '{}': id after end of list in get",
                type_name::<T>()
            );
        } else {
            match &self.nodes[id] {
                FreeListNode::Data(v) => v.clone(),
                FreeListNode::Free(_) => {
                    panic!("freelist for '{}': use after free", type_name::<T>())
                }
            }
        }
    }
    fn allocate(&mut self, t: T) -> u32 {
        let t = Arc::new(t);
        if self.head > ENFORCED_MAX {
            panic!(
                "freelist for '{}': out of allocatable ids",
                type_name::<T>()
            );
        } else if self.head < self.nodes.len() {
            let new_id = self.head;
            let new_head = match &self.nodes[new_id] {
                FreeListNode::Data(_) => panic!(
                    "freelist for '{}': __macro_internals error - attempting to allocate over value",
                    type_name::<T>(),
                ),
                FreeListNode::Free(head) => *head,
            };
            self.nodes[new_id] = FreeListNode::Data(t);
            self.head = new_head;
            new_id as u32
        } else {
            let new_id = self.head;
            self.nodes.push(FreeListNode::Data(t));
            self.head += 1;
            new_id as u32
        }
    }
    fn free(&mut self, id: u32) {
        let id = id as usize;
        if id >= self.nodes.len() {
            panic!(
                "freelist for '{}': id after end of list in free",
                type_name::<T>()
            );
        } else {
            match &self.nodes[id] {
                FreeListNode::Data(_) => {
                    self.nodes[id] = FreeListNode::Free(self.head);
                    self.head = id;
                }
                FreeListNode::Free(_) => {
                    panic!("freelist for '{}': use after free", type_name::<T>())
                }
            }
        }
    }
}

pub struct IdManager<T: Send + Sync>(RwLock<IdManagerData<T>>);
impl<T: Send + Sync> IdManager<T> {
    pub const fn new() -> Self {
        IdManager(RwLock::new(IdManagerData::new()))
    }

    pub fn get(&self, id: u32) -> Arc<T> {
        self.0.read().get(id)
    }
    pub fn allocate(&self, t: T) -> u32 {
        self.0.write().allocate(t)
    }
    pub fn free(&self, id: u32) {
        self.0.write().free(id)
    }
}
