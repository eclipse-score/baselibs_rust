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

mod heap;
mod inline;

pub use self::heap::Heap;
pub use self::inline::Inline;

use core::mem::MaybeUninit;

/// Interface to abstract over element storage kinds.
pub trait Storage<T> {
    /// Creates a new instance with enough capacity for the given number of elements.
    fn new(capacity: u32) -> Self;

    /// Returns the allocated capacity.
    fn capacity(&self) -> u32;

    /// Returns a `const` pointer to a specific element, which isn't necessarily initialized.
    ///
    /// # Safety
    ///
    /// `index < self.capacity()` must hold.
    unsafe fn element(&self, index: u32) -> &MaybeUninit<T>;

    /// Returns a `mut` pointer to a specific element, which isn't necessarily initialized.
    ///
    /// # Safety
    ///
    /// `index < self.capacity()` must hold.
    unsafe fn element_mut(&mut self, index: u32) -> &mut MaybeUninit<T>;

    /// Returns a pointer to a subslice of elements, which aren't necessarily initialized.
    ///
    /// # Safety
    ///
    /// `start <= end <= self.capacity()` must hold.
    unsafe fn subslice(&self, start: u32, end: u32) -> *const [T];

    /// Returns a pointer to a mutable subslice of elements, which aren't necessarily initialized.
    ///
    /// # Safety
    ///
    /// `start <= end <= self.capacity()` must hold.
    unsafe fn subslice_mut(&mut self, start: u32, end: u32) -> *mut [T];
}
