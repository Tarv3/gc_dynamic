use std::mem;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Evec<T> {
    pub values: Vec<Option<T>>,
    pub available: Vec<usize>,
}

impl<T> Evec<T> {
    pub fn new() -> Evec<T> {
        Evec {
            values: Vec::new(),
            available: Vec::new(),
        }
    }

    pub fn from(vec: Vec<T>) -> Evec<T> {
        let capacity = vec.capacity();
        let mut values = Vec::with_capacity(capacity);

        for value in vec {
            values.push(Some(value));
        }

        Evec {
            values,
            available: Vec::with_capacity(capacity),
        }
    }

    pub fn from_option_vec(vec: Vec<Option<T>>) -> Evec<T> {
        let mut available = Vec::with_capacity(vec.capacity());
        for (i, value) in vec.iter().enumerate() {
            if value.is_none() {
                available.push(i);
            }
        }

        Evec {
            values: vec,
            available,
        }
    }

    pub fn next_available(&self) -> usize {
        match self.available.last() {
            Some(index) => *index,
            None => self.values.len()
        }
    }

    pub fn with_capacity(capacity: usize) -> Evec<T> {
        Evec {
            values: Vec::with_capacity(capacity),
            available: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, value: T) -> usize {
        match self.available.pop() {
            Some(index) => {
                self.values[index] = Some(value);
                index
            }
            None => {
                let index = self.values.len();
                self.values.push(Some(value));
                index
            }
        }
    }

    pub fn remove(&mut self, index: usize) {
        assert!(index < self.values.len());
        if self.values[index].is_some() {
            self.values[index] = None;
            self.available.push(index);
        }
    }

    pub fn remove_unused(&mut self) {
        self.available.clear();
        let mut values = Vec::with_capacity(self.values.capacity());
        mem::swap(&mut self.values, &mut values);
        for item in values.into_iter().filter(|item| item.is_some()) {
            self.values.push(item);
        }
    }
}

impl<T> Index<usize> for Evec<T> {
    type Output = Option<T>;

    fn index(&self, index: usize) -> &Option<T> {
        &self.values[index]
    }
}

impl<T> IndexMut<usize> for Evec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Option<T> {
        &mut self.values[index]
    }
}

#[cfg(test)]
mod tests {
    use evec::Evec;
    #[test]
    fn first_tests() {
        let mut evec = Evec::from(vec![1, 2, 3, 4]);

        evec.remove(2);
        evec.push(10);
        evec.push(5);

        assert_eq!(evec[2], Some(10));
        assert_eq!(evec[4], Some(5));
    }

    #[test]
    fn remove_test() {
        let mut evec = Evec::from_option_vec(vec![Some(1), Some(2), None, None, Some(5)]);
        evec.remove_unused();

        assert_eq!(evec[2], Some(5));
    }
}
