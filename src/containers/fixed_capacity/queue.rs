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

use core::ops;

use crate::generic::queue::GenericQueue;
use crate::storage::Heap;

/// A fixed-capacity queue.
///
/// The queue can hold between 0 and `CAPACITY` elements, and behaves similarly to Rust's `VecDeque`,
/// except that it allocates memory immediately on construction, and can't shrink or grow.
pub struct FixedCapacityQueue<T> {
    inner: GenericQueue<T, Heap<T>>,
}

impl<T> FixedCapacityQueue<T> {
    /// Creates an empty queue and allocates memory for up to `capacity` elements, where `capacity <= u32::MAX`.
    ///
    /// # Panics
    ///
    /// - Panics if `capacity > u32::MAX`.
    /// - Panics if the memory allocation fails.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        assert!(capacity <= u32::MAX as usize, "FixedQueue can hold at most u32::MAX elements");
        Self {
            inner: GenericQueue::new(capacity as u32),
        }
    }
}

impl<T> Drop for FixedCapacityQueue<T> {
    fn drop(&mut self) {
        self.inner.clear();
    }
}

impl<T> ops::Deref for FixedCapacityQueue<T> {
    type Target = GenericQueue<T, Heap<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> ops::DerefMut for FixedCapacityQueue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    fn to_vec<T: Copy>((first, second): (&[T], &[T])) -> Vec<T> {
        let mut elements = first.to_vec();
        elements.extend_from_slice(second);
        elements
    }

    #[test]
    fn push_back_and_pop_front() {
        fn run_test(n: usize) {
            let mut queue = FixedCapacityQueue::<i64>::new(n);
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
            let mut queue = FixedCapacityQueue::<i64>::new(n);
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
            let mut queue = FixedCapacityQueue::<i64>::new(n);

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
