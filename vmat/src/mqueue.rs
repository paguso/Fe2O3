use std::cmp::Ordering;
use std::collections::VecDeque;

pub struct MQueueXtrIter<'a, T>
where
    T: Ord,
{
    src: &'a MQueue<T>,
    index: usize,
}

impl<'a, T> Iterator for MQueueXtrIter<'a, T>
where
    T: Ord,
{
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.src.minmax.len()
            && (self.index == 0
                || self.src.queue[self.src.minmax[self.index - 1] - self.src.popped]
                    == self.src.queue[self.src.minmax[self.index] - self.src.popped])
        {
            self.index += 1;
            return self
                .src
                .queue
                .get(self.src.minmax[self.index - 1] - self.src.popped);
        } else {
            return None;
        }
    }
}

#[derive(Clone)]
pub struct MQueue<T>
where
    T: Ord,
{
    pub queue: VecDeque<T>,
    pub minmax: VecDeque<usize>,
    crit: Ordering,
    popped: usize,
}

impl<T> MQueue<T>
where
    T: Ord,
{
    pub fn new_min() -> Self {
        MQueue {
            queue: VecDeque::new(),
            minmax: VecDeque::new(),
            crit: Ordering::Less,
            popped: 0,
        }
    }

    pub fn new_max() -> Self {
        MQueue {
            queue: VecDeque::new(),
            minmax: VecDeque::new(),
            crit: Ordering::Greater,
            popped: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, item: T) {
        let l = self.len();
        while self.minmax.len() > 0
            && item.cmp(&self.queue[self.minmax.back().unwrap() - self.popped]) == self.crit
        {
            self.minmax.pop_back();
        }
        self.queue.push_back(item);
        self.minmax.push_back(self.popped + l);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.queue.is_empty() {
            return None;
        }
        if *self.minmax.front().unwrap() == self.popped {
            self.minmax.pop_front();
        }
        self.popped += 1;
        self.queue.pop_front()
    }

    pub fn xtr(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        } else {
            return self.queue.get(self.minmax.front().unwrap() - self.popped);
        }
    }

    pub fn xtr_iter<'a>(&'a self) -> MQueueXtrIter<'a, T> {
        MQueueXtrIter {
            src: self,
            index: 0,
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
        println!("{0:?} {1:?}", queue.queue, queue.minmax);
        assert_eq!(*queue.xtr().unwrap(), 30 as u32);
        queue.push(20);
        println!("{0:?} {1:?}", queue.queue, queue.minmax);
        assert_eq!(*queue.xtr().unwrap(), 20 as u32);
        queue.push(40);
        println!("{0:?} {1:?}", queue.queue, queue.minmax);
        assert_eq!(*queue.xtr().unwrap(), 20 as u32);
        queue.pop();
        println!("{0:?} {1:?}", queue.queue, queue.minmax);
        assert_eq!(*queue.xtr().unwrap(), 20 as u32);
        queue.pop();
        println!("{0:?} {1:?}", queue.queue, queue.minmax);
        assert_eq!(*queue.xtr().unwrap(), 40 as u32);
        queue.pop();
        println!("{0:?} {1:?}", queue.queue, queue.minmax);
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

    #[test]
    fn test_xtr_iter() {
        let mut queue: MQueue<u32> = MQueue::new_min();
        queue.push(8);
        println!("q {0:?} {1:?}", queue.queue, queue.minmax);
        queue.push(5);
        println!("q {0:?} {1:?}", queue.queue, queue.minmax);
        queue.push(8);
        println!("q {0:?} {1:?}", queue.queue, queue.minmax);
        queue.push(9);
        println!("q {0:?} {1:?}", queue.queue, queue.minmax);
        queue.push(7);
        println!("q {0:?} {1:?}", queue.queue, queue.minmax);
        queue.push(5);
        println!("q {0:?} {1:?}", queue.queue, queue.minmax);
        queue.push(8);
        println!("q {0:?} {1:?}", queue.queue, queue.minmax);
        queue.push(5);
        println!("q {0:?} {1:?}", queue.queue, queue.minmax);
        let mut c = 0;
        for m in queue.xtr_iter() {
            assert_eq!(*m, 5);
            c += 1;
        }
        assert_eq!(c, 3);
        queue.pop();
        queue.pop();
        c = 0;
        for m in queue.xtr_iter() {
            assert_eq!(*m, 5);
            c += 1;
        }
        assert_eq!(c, 2);
        queue.pop();
        queue.pop();
        queue.pop();
        queue.pop();
        c = 0;
        for m in queue.xtr_iter() {
            assert_eq!(*m, 5);
            c += 1;
        }
        assert_eq!(c, 1);
        queue.pop();
        queue.pop();
        queue.push(8);
        c = 0;
        for m in queue.xtr_iter() {
            assert_eq!(*m, 8);
            c += 1;
        }
        assert_eq!(c, 1);
    }
}
