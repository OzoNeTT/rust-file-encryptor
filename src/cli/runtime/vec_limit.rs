use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VecLimited<T> {
    vec: Vec<T>,
    limit: usize,
}

impl<T> VecLimited<T> {
    pub fn new(limit: usize) -> Self {
        Self {
            vec: Vec::new(),
            limit,
        }
    }

    pub fn with_capacity(capacity: usize, limit: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
            limit,
        }
    }

    #[inline]
    pub fn get_vec(&self) -> &Vec<T> {
        &self.vec
    }

    #[inline]
    pub fn get_limit(&self) -> usize {
        self.limit
    }

    #[inline]
    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
    }

    pub fn push(&mut self, value: T) {
        if self.vec.len() >= self.limit {
            self.vec.remove(0);
        }

        self.vec.push(value);
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.vec.iter()
    }
}

impl<T, I: SliceIndex<[T]>> Index<I> for VecLimited<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.vec[index]
    }
}

impl<T, I: SliceIndex<[T]>> IndexMut<I> for VecLimited<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.vec[index]
    }
}
