

pub struct MinMaxStack<T>
where T: Ord
{
    stack: Vec<(T,usize)>
}

impl<T> MinMaxStack<T>
where T: Ord
{
    pub fn new() -> Self {
        MinMaxStack{
            stack: vec![]
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
    
    pub fn push(&mut self, item: T) {
        let l = self.len();
        if l==0 || item.ge(&self.stack[self.stack[l-1].1].0) {
            self.stack.push((item, l));
        }
        else {
            self.stack.push((item, self.stack[l-1].1));
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        match self.stack.pop() {
            Some(pair) => Some(pair.0),
            None => None
        }
    }

    pub fn max(&self) -> Option<&T> {
        if self.len() > 0 {
            Some(&self.stack[self.stack.len()-1].0)
        }
        else {
            None
        }
    }
}
    
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_push_pop() {
        let mut stack: MinMaxStack<u32> = MinMaxStack::new();
        stack.push(10);
        assert_eq!(*stack.max().unwrap(), 10 as u32); 
        stack.push(20);
        assert_eq!(*stack.max().unwrap(), 20 as u32); 
        stack.push(30);
        assert_eq!(*stack.max().unwrap(), 30 as u32); 
        stack.push(15);
        assert_eq!(*stack.max().unwrap(), 30 as u32); 
        stack.pop();
        assert_eq!(*stack.max().unwrap(), 30 as u32); 
        stack.pop();
        assert_eq!(*stack.max().unwrap(), 20 as u32); 
    }
}