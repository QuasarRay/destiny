//! Explicitly bounded lists for concrete original network scenarios.
//!
//! Capacity exhaustion is a scope error, never successful omission. The original
//! Python lists are unbounded; proofs using this type declare their scenario
//! cardinalities and do not claim correspondence beyond this capacity.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundedList<T: Copy, const N: usize> {
    values: [Option<T>; N],
    length: usize,
}

impl<T: Copy, const N: usize> Default for BoundedList<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const N: usize> BoundedList<T, N> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            values: [None; N],
            length: 0,
        }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.length
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<T> {
        if index >= self.length {
            return None;
        }
        self.values.get(index).copied().flatten()
    }

    pub fn push(&mut self, value: T) -> bool {
        if let Some(slot) = self.values.get_mut(self.length) {
            *slot = Some(value);
            self.length = self.length.saturating_add(1);
            true
        } else {
            false
        }
    }

    pub fn set(&mut self, index: usize, value: T) -> bool {
        if index >= self.length {
            return false;
        }
        if let Some(slot) = self.values.get_mut(index) {
            *slot = Some(value);
            true
        } else {
            false
        }
    }

    pub fn insert(&mut self, index: usize, value: T) -> bool {
        if index > self.length || self.length >= N {
            return false;
        }
        let mut i = self.length;
        while i > index {
            let previous = self.values.get(i.saturating_sub(1)).copied().flatten();
            if let Some(slot) = self.values.get_mut(i) {
                *slot = previous;
            }
            i = i.saturating_sub(1);
        }
        if let Some(slot) = self.values.get_mut(index) {
            *slot = Some(value);
            self.length = self.length.saturating_add(1);
            true
        } else {
            false
        }
    }

    pub fn extend(&mut self, other: &Self) -> bool {
        // Check first: a scope failure must not silently truncate a Python list.
        if other.length > N.saturating_sub(self.length) {
            return false;
        }
        let mut i = 0;
        while i < other.length {
            if let Some(value) = other.get(i) {
                if !self.push(value) {
                    return false;
                }
            } else {
                return false;
            }
            i = i.saturating_add(1);
        }
        true
    }
}
