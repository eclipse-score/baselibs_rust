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

extern crate alloc;
use alloc::alloc::alloc;
use alloc::alloc::dealloc;

use core::ptr::NonNull;

use crate::allocator_traits::{AllocationError, BasicAllocator};

#[derive(Debug, Clone, Copy)]
pub struct GlobalAllocator;

impl BasicAllocator for GlobalAllocator {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, AllocationError> {
        if layout.size() == 0 {
            return Err(AllocationError::ZeroSizeAllocation);
        }

        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                return Err(AllocationError::OutOfMemory);
            }
            Ok(NonNull::slice_from_raw_parts(
                NonNull::new_unchecked(ptr),
                layout.size(),
            ))
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
        dealloc(ptr.as_ptr(), layout);
    }
}

impl Default for GlobalAllocator {
    fn default() -> Self {
        GlobalAllocator
    }
}
