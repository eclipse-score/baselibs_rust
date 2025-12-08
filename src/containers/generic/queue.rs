// *******************************************************************************
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0
//
// SPDX-License-Identifier: Apache-2.0
// *******************************************************************************

use core::fmt;
use core::marker::PhantomData;
use core::mem::needs_drop;
use core::ops::Range;
use core::ptr;

use crate::storage::Storage;

#[repr(C)]
pub struct GenericQueue<T, S: Storage<T>> {
    /// The current number of elements in the queue.
    len: u32,
    /// The index of the next element to be returned by [`pop_front()`](Self::pop_front).
    front_index: u32,
    storage: S,
    _marker: PhantomData<T>,
}

impl<T, S: Storage<T>> GenericQueue<T, S> {
    /// Creates an empty queue.
    pub fn new(capacity: u32) -> Self {
        Self {
            len: 0,
            front_index: 0,
            storage: S::new(capacity),
            _marker: PhantomData,
        }
    }

    /// Extracts the slices containing the entire queue contents, in order.
    ///
    /// The caller should not make any assumptions about the distribution of the elements between
    /// the two slices, except for ordering.
    /// In particular, the first slice might be empty even though the second isn't.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let (first, second) = queue.as_slices();
    /// let elements: Vec<_> = std::iter::chain(first, second).collect();
    /// println!("Elements in queue: {elements:?}");
    /// ```
    pub fn as_slices(&self) -> (&[T], &[T]) {
        let (first, second) = self.slice_ranges();
        let first = unsafe { &*self.storage.subslice(first.start, first.end) };
        let second = unsafe { &*self.storage.subslice(second.start, second.end) };
        (first, second)
    }

    /// Extracts the slices containing the entire queue contents, in order.
    ///
    /// The caller should not make any assumptions about the distribution of the elements between
    /// the two slices, except for ordering.
    /// In particular, the first slice might be empty even though the second isn't.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let (first, second) = queue.as_mut_slices();
    /// for elements in std::iter::chain(first, second) {
    ///     *element *= 2;
    /// }
    /// ```
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        let (first, second) = self.slice_ranges();
        let first = unsafe { &mut *self.storage.subslice_mut(first.start, first.end) };
        let second = unsafe { &mut *self.storage.subslice_mut(second.start, second.end) };
        (first, second)
    }

    /// Returns the maximum number of elements the queue can hold.
    pub fn capacity(&self) -> usize {
        self.storage.capacity() as usize
    }

    /// Returns the current number of elements in the queue.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns `true` if and only if the queue doesn't contain any elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if and only if the queue has reached its capacity.
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Tries to push an element to the back of the queue.
    ///
    /// If the queue has spare capacity, the push succeeds and a reference to that element
    /// is returned; otherwise, `Err(QueueFull)` is returned.
    pub fn push_back(&mut self, value: T) -> Result<&mut T, QueueFull> {
        let capacity = self.storage.capacity();
        if self.len < capacity {
            let write_pos = self.front_index as u64 + self.len as u64;
            let write_pos = if write_pos < capacity as u64 {
                write_pos as u32
            } else {
                (write_pos - capacity as u64) as u32
            };
            self.len += 1;
            Ok(unsafe { self.storage.element_mut(write_pos).write(value) })
        } else {
            Err(QueueFull)
        }
    }

    /// Tries to push an element to the front of the queue.
    ///
    /// If the queue has spare capacity, the push succeeds and a reference to that element
    /// is returned; otherwise, `Err(QueueFull)` is returned.
    pub fn push_front(&mut self, value: T) -> Result<&mut T, QueueFull> {
        let capacity = self.storage.capacity();
        if self.len < capacity {
            let write_pos = if self.front_index > 0 { self.front_index - 1 } else { capacity - 1 };
            let element = unsafe { self.storage.element_mut(write_pos).write(value) };
            self.len += 1;
            self.front_index = write_pos;
            Ok(element)
        } else {
            Err(QueueFull)
        }
    }

    /// Tries to pop an element from the front of the queue.
    ///
    /// If the queue has at least one element, the pop succeeds; otherwise, `None` is returned.
    pub fn pop_front(&mut self) -> Option<T> {
        if self.len > 0 {
            let element = unsafe { self.storage.element(self.front_index).assume_init_read() };
            self.len -= 1;
            if self.front_index < self.storage.capacity() - 1 {
                self.front_index += 1;
            } else {
                self.front_index = 0;
            }
            Some(element)
        } else {
            None
        }
    }

    /// Tries to pop an element from the back of the queue.
    ///
    /// If the queue has at least one element, the pop succeeds; otherwise, `None` is returned.
    pub fn pop_back(&mut self) -> Option<T> {
        let capacity = self.storage.capacity();
        if self.len > 0 {
            let read_pos = self.front_index as u64 + (self.len as u64 - 1);
            let read_pos = if read_pos < capacity as u64 {
                read_pos as u32
            } else {
                (read_pos - capacity as u64) as u32
            };
            self.len -= 1;
            Some(unsafe { self.storage.element(read_pos).assume_init_read() })
        } else {
            None
        }
    }

    /// Clears the queue, removing all values.
    pub fn clear(&mut self) {
        let (first, second) = self.slice_ranges();
        // Mark queue as empty before dropping elements, to prevent double-drop in case there's a panic in drop_in_place
        self.len = 0;
        self.front_index = 0;
        if needs_drop::<T>() {
            unsafe {
                ptr::drop_in_place(self.storage.subslice_mut(first.start, first.end));
                ptr::drop_in_place(self.storage.subslice_mut(second.start, second.end));
            }
        }
    }

    fn slice_ranges(&self) -> (Range<u32>, Range<u32>) {
        // Cast to u64 to avoid overflow
        let end = self.front_index as u64 + self.len as u64;
        let capacity = self.storage.capacity();
        if end > capacity as u64 {
            let end = (end - capacity as u64) as u32;
            (self.front_index..capacity, 0..end)
        } else {
            let end = end as u32;
            (self.front_index..end, end..end)
        }
    }
}

/// Indicates that an operation failed because the queue would exceed its maximum capacity.
#[derive(Clone, Copy, Default, Debug)]
pub struct QueueFull;

impl fmt::Display for QueueFull {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "queue is full")
    }
}

impl core::error::Error for QueueFull {}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, mem::MaybeUninit};

    use super::*;

    fn to_vec<T: Copy>((first, second): (&[T], &[T])) -> Vec<T> {
        let mut elements = first.to_vec();
        elements.extend_from_slice(second);
        elements
    }

    #[test]
    fn push_back_and_pop_front() {
        fn run_test(n: usize) {
            let mut queue = GenericQueue::<i64, Vec<MaybeUninit<i64>>>::new(n as u32);
            let mut control = VecDeque::new();

            // Completely fill and empty the queue n times, but move the internal start point
            // ahead by one each time
            for _ in 0..n {
                let result = queue.pop_front();
                assert_eq!(result, None);

                for i in 0..n {
                    let value = i as i64 * 123 + 456;
                    let result = queue.push_back(value);
                    assert_eq!(*result.unwrap(), value);
                    control.push_back(value);
                    assert_eq!(to_vec(queue.as_slices()), to_vec(control.as_slices()));
                }

                let result = queue.push_back(123456);
                assert!(result.is_err());

                for _ in 0..n {
                    let expected = control.pop_front().unwrap();
                    let actual = queue.pop_front();
                    assert_eq!(actual, Some(expected));
                }

                let result = queue.pop_front();
                assert_eq!(result, None);

                // One push and one pop to move the internal start point ahead
                queue.push_back(987).unwrap();
                assert_eq!(queue.pop_front(), Some(987));
            }
        }

        for i in 0..6 {
            run_test(i);
        }
    }

    #[test]
    fn push_front_and_pop_back() {
        fn run_test(n: usize) {
            let mut queue = GenericQueue::<i64, Vec<MaybeUninit<i64>>>::new(n as u32);
            let mut control = VecDeque::new();

            // Completely fill and empty the queue n times, but move the internal start point
            // ahead by one each time
            for _ in 0..n {
                let result = queue.pop_back();
                assert_eq!(result, None);

                for i in 0..n {
                    let value = i as i64 * 123 + 456;
                    let result = queue.push_front(value);
                    assert_eq!(*result.unwrap(), value);
                    control.push_front(value);
                    assert_eq!(to_vec(queue.as_slices()), to_vec(control.as_slices()));
                }

                let result = queue.push_front(123456);
                assert!(result.is_err());

                for _ in 0..n {
                    let expected = control.pop_back().unwrap();
                    let actual = queue.pop_back();
                    assert_eq!(actual, Some(expected));
                }

                let result = queue.pop_back();
                assert_eq!(result, None);

                // One push and one pop to move the internal start point ahead
                queue.push_front(987).unwrap();
                assert_eq!(queue.pop_back(), Some(987));
            }
        }

        for i in 0..6 {
            run_test(i);
        }
    }

    #[test]
    fn is_empty_and_is_full() {
        fn run_test(n: usize) {
            let mut queue = GenericQueue::<i64, Vec<MaybeUninit<i64>>>::new(n as u32);

            // Completely fill and empty the queue n times, but move the internal start point
            // ahead by one each time
            for _ in 0..n {
                assert!(queue.is_empty());

                for i in 0..n {
                    assert!(!queue.is_full());
                    queue.push_back(i as i64 * 123 + 456).unwrap();
                    assert!(!queue.is_empty());
                }

                assert!(queue.is_full());

                for _ in 0..n {
                    assert!(!queue.is_empty());
                    queue.pop_front();
                    assert!(!queue.is_full());
                }

                assert!(queue.is_empty());

                // One push and one pop to move the internal start point ahead
                queue.push_back(987).unwrap();
                assert_eq!(queue.pop_front(), Some(987));
            }
        }

        for i in 0..6 {
            run_test(i);
        }
    }
}
