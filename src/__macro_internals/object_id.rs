use crate::errors::*;
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

    fn get(&self, id: u32) -> Result<Arc<T>> {
        let type_name = type_name::<T>();
        let id = id as usize;
        if id >= self.nodes.len() {
            jni_bail!("freelist for '{type_name}': id after end of list in get");
        } else {
            match &self.nodes[id] {
                FreeListNode::Data(v) => Ok(v.clone()),
                FreeListNode::Free(_) => jni_bail!("freelist for '{type_name}': use after free"),
            }
        }
    }
    fn allocate(&mut self, t: T) -> Result<u32> {
        let type_name = type_name::<T>();
        let t = Arc::new(t);
        if self.head > ENFORCED_MAX {
            jni_bail!("freelist for '{type_name}': out of allocatable ids");
        } else if self.head < self.nodes.len() {
            let new_id = self.head;
            let new_head = match &self.nodes[new_id] {
                FreeListNode::Data(_) => jni_bail!("freelist for '{type_name}': already allocated"),
                FreeListNode::Free(head) => *head,
            };
            self.nodes[new_id] = FreeListNode::Data(t);
            self.head = new_head;
            Ok(new_id as u32)
        } else {
            let new_id = self.head;
            self.nodes.push(FreeListNode::Data(t));
            self.head += 1;
            Ok(new_id as u32)
        }
    }
    fn free(&mut self, id: u32) -> Result<()> {
        let type_name = type_name::<T>();
        let id = id as usize;
        if id >= self.nodes.len() {
            jni_bail!("freelist for '{type_name}': id after end of list in free");
        } else {
            match &self.nodes[id] {
                FreeListNode::Data(_) => {
                    self.nodes[id] = FreeListNode::Free(self.head);
                    self.head = id;
                    Ok(())
                }
                FreeListNode::Free(_) => jni_bail!("freelist for '{type_name}': use after free"),
            }
        }
    }
}

pub struct IdManager<T: Send + Sync>(RwLock<IdManagerData<T>>);
impl<T: Send + Sync> IdManager<T> {
    pub const fn new() -> Self {
        IdManager(RwLock::new(IdManagerData::new()))
    }

    pub fn get(&self, id: u32) -> Result<Arc<T>> {
        self.0.read().get(id)
    }
    pub fn allocate(&self, t: T) -> Result<u32> {
        self.0.write().allocate(t)
    }
    pub fn free(&self, id: u32) -> Result<()> {
        self.0.write().free(id)
    }
}
