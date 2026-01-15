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
use core::ops;

use crate::generic::vec::GenericVec;
use crate::storage::Heap;

/// A fixed-capacity vector.
///
/// The vector can hold between 0 and `CAPACITY` elements, and behaves similarly to Rust's `Vec`,
/// except that it allocates memory immediately on construction, and can't shrink or grow.
pub struct FixedCapacityVec<T> {
    inner: GenericVec<T, Heap<T>>,
}

/// # req-Id: comp_req__containers_rust__fixed_vector
impl<T> FixedCapacityVec<T> {
    /// Creates an empty vector and allocates memory for up to `capacity` elements, where `capacity <= u32::MAX`.
    ///
    /// # Panics
    ///
    /// - Panics if `capacity > u32::MAX`.
    /// - Panics if the memory allocation fails.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        assert!(
            capacity <= u32::MAX as usize,
            "FixedCapacityVec can hold at most u32::MAX elements"
        );
        Self {
            inner: GenericVec::new(capacity as u32),
        }
    }

    /// Tries to create an empty vector for up to `capacity` elements, where `capacity <= u32::MAX`.
    ///
    /// Returns `None` if `capacity > u32::MAX`, or if the memory allocation fails.
    #[must_use]
    pub fn try_new(capacity: usize) -> Option<Self> {
        if capacity <= u32::MAX as usize {
            Some(Self {
                inner: GenericVec::try_new(capacity as u32)?,
            })
        } else {
            None
        }
    }
}

impl<T> Drop for FixedCapacityVec<T> {
    fn drop(&mut self) {
        self.inner.clear();
    }
}

impl<T> ops::Deref for FixedCapacityVec<T> {
    type Target = GenericVec<T, Heap<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> ops::DerefMut for FixedCapacityVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: fmt::Debug> fmt::Debug for FixedCapacityVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_slice(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_pop() {
        fn run_test(n: usize) {
            let mut vector = FixedCapacityVec::<i64>::new(n);
            let mut control = vec![];

            let result = vector.pop();
            assert_eq!(result, None);

            for i in 0..n {
                let value = i as i64 * 123 + 456;
                let result = vector.push(value);
                assert_eq!(*result.unwrap(), value);
                control.push(value);
                assert_eq!(vector.as_slice(), control.as_slice());
            }

            let result = vector.push(123456);
            assert!(result.is_err());

            for _ in 0..n {
                let expected = control.pop().unwrap();
                let actual = vector.pop();
                assert_eq!(actual, Some(expected));
            }

            let result = vector.pop();
            assert_eq!(result, None);
        }

        for i in 0..6 {
            run_test(i);
        }
    }

    #[test]
    fn is_full_and_is_empty() {
        fn run_test(n: usize) {
            let mut vector = FixedCapacityVec::<i64>::new(n);
            assert!(vector.is_empty());

            for i in 0..n {
                assert!(!vector.is_full());
                vector.push(i as i64 * 123 + 456).unwrap();
                assert!(!vector.is_empty());
            }

            assert!(vector.is_full());

            for _ in 0..n {
                assert!(!vector.is_empty());
                vector.pop();
                assert!(!vector.is_full());
            }

            assert!(vector.is_empty());
        }

        for i in 0..6 {
            run_test(i);
        }
    }
}
