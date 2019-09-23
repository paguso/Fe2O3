use crate::mstack::MStack;
use std::cmp::Ordering;

#[derive(Clone)]
pub struct MQueue<T>
where
    T: Ord,
{
    front: MStack<T>,
    rear: MStack<T>,
    crit: Ordering,
}

impl<T> MQueue<T>
where
    T: Ord,
{
    pub fn new_min() -> Self {
        MQueue {
            front: MStack::new_min(),
            rear: MStack::new_min(),
            crit: Ordering::Less,
        }
    }

    pub fn new_max() -> Self {
        MQueue {
            front: MStack::new_max(),
            rear: MStack::new_max(),
            crit: Ordering::Greater,
        }
    }

    pub fn len(&self) -> usize {
        self.front.len() + self.rear.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, item: T) {
        self.rear.push(item)
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.front.is_empty() {
            while !self.rear.is_empty() {
                self.front.push(self.rear.pop().unwrap());
            }
        }
        self.front.pop()
    }

    pub fn xtr(&self) -> Option<&T> {
        if self.front.is_empty() {
            self.rear.xtr()
        } else if self.rear.is_empty() {
            self.front.xtr()
        } else {
            let f = self.front.xtr();
            let r = self.rear.xtr();
            if f.unwrap().cmp(r.unwrap()) == self.crit {
                f
            } else {
                r
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop_min() {
        let mut queue: MQueue<u32> = MQueue::new_min();
        queue.push(30);
        assert_eq!(*queue.xtr().unwrap(), 30 as u32);
        queue.push(20);
        assert_eq!(*queue.xtr().unwrap(), 20 as u32);
        queue.push(40);
        assert_eq!(*queue.xtr().unwrap(), 20 as u32);
        queue.pop();
        assert_eq!(*queue.xtr().unwrap(), 20 as u32);
        queue.pop();
        assert_eq!(*queue.xtr().unwrap(), 40 as u32);
        queue.pop();
        assert!(queue.pop().is_none());
    }

    #[test]
    fn test_push_pop_max() {
        let mut queue: MQueue<u32> = MQueue::new_max();
        queue.push(30);
        assert_eq!(*queue.xtr().unwrap(), 30 as u32);
        queue.push(20);
        assert_eq!(*queue.xtr().unwrap(), 30 as u32);
        queue.push(40);
        assert_eq!(*queue.xtr().unwrap(), 40 as u32);
        queue.pop();
        assert_eq!(*queue.xtr().unwrap(), 40 as u32);
        queue.pop();
        assert_eq!(*queue.xtr().unwrap(), 40 as u32);
        queue.push(15);
        assert_eq!(*queue.xtr().unwrap(), 40 as u32);
        queue.push(55);
        assert_eq!(*queue.xtr().unwrap(), 55 as u32);
    }

}
