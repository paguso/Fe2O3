use std::cmp::Ordering;

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
}
