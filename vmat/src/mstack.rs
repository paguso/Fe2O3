use std::cmp::Ordering;

pub struct MStackXtrIter<'a, T>
where
    T: Ord,
{
    src: &'a MStack<T>,
    index: usize,
}

impl<'a, T> Iterator for MStackXtrIter<'a, T>
where
    T: Ord,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            return None;
        }
        if self.index == self.src.len() {
            if self.src.stack[self.index - 1].0
                == self.src.stack[self.src.stack[self.index - 1].1].0
            {
                // top is min/max
                self.index -= 1;
            } else {
                self.index = self.src.stack[self.src.len() - 1].1;
            }
            return Some(&self.src.stack[self.index].0);
        } else if self.src.stack[self.index].1 != self.index {
            self.index = self.src.stack[self.index].1;
            return Some(&self.src.stack[self.index].0);
        } else {
            return None;
        }
    }
}

#[derive(Clone)]
pub struct MStack<T>
where
    T: Ord,
{
    stack: Vec<(T, usize)>,
    crit: Ordering,
}

impl<T> MStack<T>
where
    T: Ord,
{
    pub fn new_max() -> Self {
        MStack {
            stack: vec![],
            crit: Ordering::Greater,
        }
    }

    pub fn new_min() -> Self {
        MStack {
            stack: vec![],
            crit: Ordering::Less,
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.len() == 0
    }

    pub fn push(&mut self, item: T) {
        let l = self.len();
        if l == 0 || item.cmp(&self.stack[self.stack[l - 1].1].0) == self.crit {
            self.stack.push((item, l));
        } else if self.stack[self.stack[l - 1].1].0 == self.stack[l - 1].0 {
            self.stack.push((item, l - 1));
        } else {
            self.stack.push((item, self.stack[l - 1].1));
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        match self.stack.pop() {
            Some(pair) => Some(pair.0),
            None => None,
        }
    }

    pub fn xtr(&self) -> Option<&T> {
        if self.len() > 0 {
            Some(&self.stack[self.stack[self.stack.len() - 1].1].0)
        } else {
            None
        }
    }

    pub fn all_xtr_iter<'a>(&'a self) -> MStackXtrIter<'a, T> {
        MStackXtrIter {
            src: &self,
            index: self.len(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_push_pop_min() {
        let mut stack: MStack<u32> = MStack::new_min();
        stack.push(20);
        assert_eq!(*stack.xtr().unwrap(), 20 as u32);
        stack.push(10);
        assert_eq!(*stack.xtr().unwrap(), 10 as u32);
        stack.push(30);
        assert_eq!(*stack.xtr().unwrap(), 10 as u32);
        stack.push(5);
        assert_eq!(*stack.xtr().unwrap(), 5 as u32);
        stack.pop();
        assert_eq!(*stack.xtr().unwrap(), 10 as u32);
        stack.pop();
        assert_eq!(*stack.xtr().unwrap(), 10 as u32);
        stack.pop();
        assert_eq!(*stack.xtr().unwrap(), 20 as u32);
        stack.pop();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_push_pop_max() {
        let mut stack: MStack<u32> = MStack::new_max();
        stack.push(10);
        assert_eq!(*stack.xtr().unwrap(), 10 as u32);
        stack.push(20);
        assert_eq!(*stack.xtr().unwrap(), 20 as u32);
        stack.push(30);
        assert_eq!(*stack.xtr().unwrap(), 30 as u32);
        stack.push(15);
        assert_eq!(*stack.xtr().unwrap(), 30 as u32);
        stack.pop();
        assert_eq!(*stack.xtr().unwrap(), 30 as u32);
        stack.pop();
        assert_eq!(*stack.xtr().unwrap(), 20 as u32);
        stack.pop();
        assert_eq!(*stack.xtr().unwrap(), 10 as u32);
        stack.pop();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_all_xstr() {
        let mut stack: MStack<u32> = MStack::new_min();
        for m in stack.all_xtr_iter() {
            panic!("min stack is empty");
        }
        stack.push(6);
        stack.push(5);
        stack.push(7);
        stack.push(8);
        stack.push(5);
        stack.push(8);
        stack.push(5);
        stack.push(8);
        let mut count = 0;
        for m in stack.all_xtr_iter() {
            assert_eq!(*m, 5 as u32);
            count += 1;
        }
        assert_eq!(count, 3);
        stack.pop();
        stack.pop();
        stack.pop();
        count = 0;
        for m in stack.all_xtr_iter() {
            assert_eq!(*m, 5 as u32);
            count += 1;
        }
        assert_eq!(count, 2);
    }
}
