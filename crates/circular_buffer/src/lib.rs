use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct CircularBuffer<T> {
    buffer: VecDeque<T>,
    max_size: usize,
}

impl<T> CircularBuffer<T> {
    pub fn new(max_size: usize) -> CircularBuffer<T> {
        CircularBuffer {
            buffer: VecDeque::with_capacity(max_size),
            max_size,
        }
    }
    
    pub fn push_front(&mut self, value: T) {
        if self.buffer.len() == self.max_size {
            self.buffer.pop_back();
        }
        self.buffer.push_front(value);
    }
    
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    pub fn capacity(&self) -> usize {
        self.max_size
    }
    
    pub fn iter(&self) -> std::collections::vec_deque::Iter<T> {
        self.buffer.iter()
    }
    
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
    
    pub fn first(&self) -> Option<&T> {
        self.buffer.front()
    }
    
    pub fn last(&self) -> Option<&T> {
        self.buffer.back()
    }
    
    pub fn nth_from_front(&self, n: usize) -> Option<&T> {
        self.buffer.get(n)
    }
    
}
