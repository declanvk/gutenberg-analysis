use std::mem;
use std::ops::Index;
use std::iter::{Extend, Iterator, Zip};
use std::vec::IntoIter;

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SparseVector<E> {
    // Dimension of the vector
    size: usize,
    // Number of non-zero elements
    nnz: usize,
    // Non-zero elements
    data: Vec<E>,
    // Indices of non-zero elements
    index: Vec<u32>,
    // Capacity of data and index
    default: E,
}

impl<E> SparseVector<E> {
    pub fn new(size: usize, default: E) -> Self {
        SparseVector {
            size,
            nnz: 0,
            data: Vec::new(),
            index: Vec::new(),
            default,
        }
    }

    pub fn with_capacity(size: usize, default: E, capacity: usize) -> Self {
        SparseVector {
            size,
            nnz: 0,
            data: Vec::with_capacity(capacity),
            index: Vec::with_capacity(capacity),
            default,
        }
    }

    pub fn contains(&self, index: u32) -> bool {
        self.get_internal_index(index).is_some()
    }

    pub fn get_internal_index(&self, index: u32) -> Option<usize> {
        match self.index.binary_search(&index) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    pub fn get(&self, index: u32) -> Option<&E> {
        match self.get_internal_index(index) {
            Some(internal_idx) => Some(&self.data[internal_idx]),
            None => None,
        }
    }

    pub fn get_mut(&mut self, index: u32) -> Option<&mut E> {
        match self.get_internal_index(index) {
            Some(internal_idx) => Some(&mut self.data[internal_idx]),
            None => None,
        }
    }

    pub fn insert(&mut self, index: u32, elem: E) -> Option<E> {
        self.insert_with_merge(index, elem, &mut |old_elem, mut new_elem| {
            mem::swap(old_elem, &mut new_elem);

            new_elem
        })
    }

    pub fn insert_with_merge<F>(&mut self, index: u32, elem: E, merge: &mut F) -> Option<E>
    where
        F: FnMut(&mut E, E) -> E,
    {
        assert!(index < self.size as u32);

        match self.index.binary_search(&index) {
            Ok(idx) => {
                // Index found in list
                let old_value = merge(self.data.get_mut(idx).unwrap(), elem);

                Some(old_value)
            }
            Err(idx) => {
                // Index not found in list
                debug_assert!(self.index.len() != self.size);
                self.data.insert(idx, elem);
                self.index.insert(idx, index);

                self.nnz += 1;

                None
            }
        }
    }

    pub fn extend_with_merge<T, F>(&mut self, iter: T, mut merge: F)
    where
        T: IntoIterator<Item = (u32, E)>,
        F: FnMut(&mut E, E) -> E,
    {
        for (index, elem) in iter {
            self.insert_with_merge(index, elem, &mut merge);
        }
    }

    pub fn zip_nonzero_pairs<'a, 'b, B: 'b>(
        &'a self,
        other: &'b SparseVector<B>,
    ) -> NonZeroPairs<'a, 'b, E, B> {
        NonZeroPairs {
            index_slice_a: self.index.as_slice(),
            data_slice_a: self.data.as_slice(),
            ptr_a: 0,

            index_slice_b: other.index.as_slice(),
            data_slice_b: other.data.as_slice(),
            ptr_b: 0,
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, E> {
        Iter {
            index_slice: self.index.as_slice(),
            data_slice: self.data.as_slice(),
            ptr: 0,
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, E> {
        IterMut {
            index_slice: self.index.as_slice(),
            data_slice: self.data.as_mut(),
            ptr: 0,
        }
    }
}

impl<E> Index<u32> for SparseVector<E> {
    type Output = E;

    fn index(&self, index: u32) -> &Self::Output {
        match self.get(index) {
            Some(val) => val,
            None => &self.default,
        }
    }
}

impl<E> Extend<(u32, E)> for SparseVector<E> {
    fn extend<T: IntoIterator<Item = (u32, E)>>(&mut self, iter: T) {
        for (index, elem) in iter {
            self.insert(index, elem);
        }
    }
}

#[derive(Debug)]
pub struct Iter<'a, E: 'a> {
    index_slice: &'a [u32],
    data_slice: &'a [E],
    ptr: usize,
}

impl<'a, E> Iterator for Iter<'a, E> {
    type Item = (u32, &'a E);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.index_slice.len() {
            None
        } else {
            let result = (self.index_slice[self.ptr], &self.data_slice[self.ptr]);
            self.ptr += 1;

            Some(result)
        }
    }
}

#[derive(Debug)]
pub struct IterMut<'a, E: 'a> {
    index_slice: &'a [u32],
    data_slice: &'a mut [E],
    ptr: usize,
}

impl<'a, E> Iterator for IterMut<'a, E> {
    type Item = (u32, &'a mut E);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.index_slice.len() {
            None
        } else {
            let result = (self.index_slice[self.ptr], &mut self.data_slice[self.ptr]);
            self.ptr += 1;

            Some(result)
        }
    }
}

impl<E> IntoIterator for SparseVector<E> {
    type Item = (u32, E);
    type IntoIter = Zip<IntoIter<u32>, IntoIter<E>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.index.into_iter().zip(self.data.into_iter())
    }
}

impl<'a, E: 'a> IntoIterator for &'a SparseVector<E> {
    type Item = (u32, &'a E);
    type IntoIter = Iter<'a, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, E: 'a> IntoIterator for &'a mut SparseVector<E> {
    type Item = (u32, &'a mut E);
    type IntoIter = IterMut<'a, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[derive(Debug)]
pub struct NonZeroPairs<'a, 'b, A: 'a, B: 'b> {
    index_slice_a: &'a [u32],
    data_slice_a: &'a [A],
    ptr_a: usize,

    index_slice_b: &'b [u32],
    data_slice_b: &'b [B],
    ptr_b: usize,
}

impl<'a, 'b, A: 'a, B: 'b> Iterator for NonZeroPairs<'a, 'b, A, B> {
    type Item = (&'a A, &'b B);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr_a >= self.index_slice_a.len() || self.ptr_b >= self.index_slice_b.len() {
            None
        } else {
            loop {
                let index_a = self.index_slice_a[self.ptr_a];
                let index_b = self.index_slice_b[self.ptr_b];

                if index_a == index_b {
                    let value_a = &self.data_slice_a[self.ptr_a];
                    let value_b = &self.data_slice_b[self.ptr_b];

                    self.ptr_a += 1;
                    self.ptr_b += 1;
                    break Some((value_a, value_b));
                } else if index_a < index_b {
                    self.ptr_a += 1;
                } else {
                    self.ptr_b += 1;
                }

                if self.ptr_a >= self.index_slice_a.len() || self.ptr_b >= self.index_slice_b.len()
                {
                    break None;
                }
            }
        }
    }
}
