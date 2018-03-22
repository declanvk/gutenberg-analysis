use alloc::raw_vec::RawVec;
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::mem;
use std::ptr;
use std::fmt;
use std::ops::Index;
use std::iter::{Extend, Iterator};

pub struct SparseVector<E> {
    // Dimension of the vector
    size: usize,
    // Number of non-zero elements
    nnz: usize,
    // Non-zero elements
    data: RawVec<E>,
    // Indices of non-zero elements
    index: RawVec<u32>,
    // Capacity of data and index
    capacity: usize,
    default: E,
}

impl<E> SparseVector<E> {
    pub fn new(size: usize, default: E) -> Self {
        SparseVector {
            size,
            nnz: 0,
            data: RawVec::new(),
            index: RawVec::new(),
            capacity: 0,
            default,
        }
    }

    pub fn with_capacity(size: usize, default: E, capacity: usize) -> Self {
        SparseVector {
            size,
            nnz: 0,
            data: RawVec::with_capacity(capacity),
            index: RawVec::with_capacity(capacity),
            capacity,
            default,
        }
    }

    pub fn contains(&self, index: u32) -> bool {
        let index_slice: &[u32] = unsafe { self.index_slice() };

        match index_slice.binary_search(&index) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn get_internal_index(&self, index: u32) -> Option<usize> {
        let index_slice: &[u32] = unsafe { self.index_slice() };

        match index_slice.binary_search(&index) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    pub fn get(&self, index: u32) -> Option<&E> {
        match self.get_internal_index(index) {
            Some(internal_idx) => {
                let data_slice = unsafe { self.data_slice() };

                Some(&data_slice[internal_idx])
            }
            None => None,
        }
    }

    pub fn get_mut(&mut self, index: u32) -> Option<&mut E> {
        match self.get_internal_index(index) {
            Some(internal_idx) => {
                let data_slice = unsafe { self.data_slice_mut() };

                Some(&mut data_slice[internal_idx])
            }
            None => None,
        }
    }

    pub fn insert(&mut self, index: u32, elem: E) -> Option<E> {
        assert!(index < self.size as u32);

        if self.nnz == self.capacity && self.capacity != self.size {
            if 2 * self.capacity > self.size {
                let needed_cap = self.size - self.capacity;

                self.data.reserve_exact(self.capacity, needed_cap);
                self.index.reserve_exact(self.capacity, needed_cap);
                self.capacity = self.size;
            } else {
                self.data.double();
                self.index.double();
                self.capacity *= 2;
            }
        }

        let index_slice: &mut [u32] = unsafe { self.index_slice_mut() };
        match index_slice.binary_search(&index) {
            Ok(idx) => {
                // Index found in list
                let data_slice: &mut [E] = unsafe { self.data_slice_mut() };
                let old_value = mem::replace(&mut data_slice[idx], elem);

                Some(old_value)
            }
            Err(idx) => {
                // Index not found in list
                debug_assert!(self.capacity != self.size);
                let data_slice: &mut [E] = unsafe { self.data_slice_mut() };

                unsafe {
                    let index_ptr = index_slice.as_mut_ptr().offset(idx as isize);
                    let data_ptr = data_slice.as_mut_ptr().offset(idx as isize);

                    // Shift everything over to make space. (Duplicating the
                    // `index`th element into two consecutive places.)
                    ptr::copy(index_ptr, index_ptr.offset(1), self.nnz - idx as usize);
                    ptr::copy(data_ptr, data_ptr.offset(1), self.nnz - idx as usize);

                    ptr::write(index_ptr, index);
                    ptr::write(data_ptr, elem);
                }

                self.nnz += 1;

                None
            }
        }
    }

    pub fn zip_nonzero<'a, 'b, B: 'b>(
        &'a self,
        other: &'b SparseVector<B>,
    ) -> NonZeroElemZipIter<'a, 'b, E, B> {
        NonZeroElemZipIter {
            index_slice_a: unsafe { self.index_slice() },
            data_slice_a: unsafe { self.data_slice() },
            ptr_a: 0,

            index_slice_b: unsafe { other.index_slice() },
            data_slice_b: unsafe { other.data_slice() },
            ptr_b: 0,
        }
    }

    pub fn zip_nonzero_pairs<'a, 'b, B: 'b>(
        &'a self,
        other: &'b SparseVector<B>,
    ) -> NonZeroPairZipIter<'a, 'b, E, B> {
        NonZeroPairZipIter {
            index_slice_a: unsafe { self.index_slice() },
            data_slice_a: unsafe { self.data_slice() },
            ptr_a: 0,

            index_slice_b: unsafe { other.index_slice() },
            data_slice_b: unsafe { other.data_slice() },
            ptr_b: 0,
        }
    }

    unsafe fn data_slice(&self) -> &[E] {
        from_raw_parts(self.data.ptr(), self.nnz)
    }

    unsafe fn data_slice_mut(&self) -> &mut [E] {
        from_raw_parts_mut(self.data.ptr(), self.nnz)
    }

    unsafe fn index_slice(&self) -> &[u32] {
        from_raw_parts(self.index.ptr(), self.nnz)
    }

    unsafe fn index_slice_mut(&self) -> &mut [u32] {
        from_raw_parts_mut(self.index.ptr(), self.nnz)
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

impl<E> fmt::Debug for SparseVector<E>
where
    E: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            f.debug_list()
                .entries(self.index_slice().iter().zip(self.data_slice().iter()))
                .finish()
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
pub struct NonZeroElemZipIter<'a, 'b, A: 'a, B: 'b> {
    index_slice_a: &'a [u32],
    data_slice_a: &'a [A],
    ptr_a: usize,

    index_slice_b: &'b [u32],
    data_slice_b: &'b [B],
    ptr_b: usize,
}

impl<'a, 'b, A: 'a, B: 'b> Iterator for NonZeroElemZipIter<'a, 'b, A, B> {
    type Item = (Option<&'a A>, Option<&'b B>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr_a >= self.index_slice_a.len() || self.ptr_b >= self.index_slice_b.len() {
            None
        } else {
            let index_a = self.index_slice_a[self.ptr_a];
            let index_b = self.index_slice_b[self.ptr_b];

            if index_a == index_b {
                let value_a = &self.data_slice_a[self.ptr_a];
                let value_b = &self.data_slice_b[self.ptr_b];

                self.ptr_a += 1;
                self.ptr_b += 1;
                Some((Some(value_a), Some(value_b)))
            } else if index_a < index_b {
                let value_a = &self.data_slice_a[self.ptr_a];

                self.ptr_a += 1;
                Some((Some(value_a), None))
            } else {
                let value_b = &self.data_slice_b[self.ptr_b];

                self.ptr_b += 1;
                Some((None, Some(value_b)))
            }
        }
    }
}

#[derive(Debug)]
pub struct NonZeroPairZipIter<'a, 'b, A: 'a, B: 'b> {
    index_slice_a: &'a [u32],
    data_slice_a: &'a [A],
    ptr_a: usize,

    index_slice_b: &'b [u32],
    data_slice_b: &'b [B],
    ptr_b: usize,
}

impl<'a, 'b, A: 'a, B: 'b> Iterator for NonZeroPairZipIter<'a, 'b, A, B> {
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
