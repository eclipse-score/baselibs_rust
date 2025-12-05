//
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
//

//! Implementation of `ScoreDisplay` and `ScoreDebug` for basic types.

use crate::fmt;
use crate::fmt::*;
use crate::fmt_spec::FormatSpec;

macro_rules! impl_fmt_for_t {
    ($t:ty, $fn:ident, $($fmt:ident),*) => {
        $(
        impl $fmt for $t {
            fn fmt(&self, f: Writer, spec: &FormatSpec) -> fmt::Result {
                f.$fn(self, spec)
            }
        }
        )*
    };
}

impl_fmt_for_t!(bool, write_bool, ScoreDebug, ScoreDisplay);
impl_fmt_for_t!(f32, write_f32, ScoreDebug, ScoreDisplay);
impl_fmt_for_t!(f64, write_f64, ScoreDebug, ScoreDisplay);
impl_fmt_for_t!(i8, write_i8, ScoreDebug, ScoreDisplay);
impl_fmt_for_t!(i16, write_i16, ScoreDebug, ScoreDisplay);
impl_fmt_for_t!(i32, write_i32, ScoreDebug, ScoreDisplay);
impl_fmt_for_t!(i64, write_i64, ScoreDebug, ScoreDisplay);
impl_fmt_for_t!(u8, write_u8, ScoreDebug, ScoreDisplay);
impl_fmt_for_t!(u16, write_u16, ScoreDebug, ScoreDisplay);
impl_fmt_for_t!(u32, write_u32, ScoreDebug, ScoreDisplay);
impl_fmt_for_t!(u64, write_u64, ScoreDebug, ScoreDisplay);
impl_fmt_for_t!(&str, write_str, ScoreDisplay);

impl ScoreDebug for &str {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> fmt::Result {
        f.write_str("\"", spec)?;
        f.write_str(self, spec)?;
        f.write_str("\"", spec)
    }
}

macro_rules! impl_fmt_for_t_casted {
    ($ti:ty, $to:ty, $fn:ident, $($fmt:ident),*) => {
        $(
        impl $fmt for $ti {
            fn fmt(&self, f: Writer, spec: &FormatSpec) -> fmt::Result {
                let casted = <$to>::try_from(*self).map_err(|_| fmt::Error)?;
                f.$fn(&casted, spec)
            }
        }
        )*
    };
}

#[cfg(target_pointer_width = "32")]
impl_fmt_for_t_casted!(isize, i32, write_i32, ScoreDebug, ScoreDisplay);
#[cfg(target_pointer_width = "64")]
impl_fmt_for_t_casted!(isize, i64, write_i64, ScoreDebug, ScoreDisplay);
#[cfg(target_pointer_width = "32")]
impl_fmt_for_t_casted!(usize, u32, write_u32, ScoreDebug, ScoreDisplay);
#[cfg(target_pointer_width = "64")]
impl_fmt_for_t_casted!(usize, u64, write_u64, ScoreDebug, ScoreDisplay);

#[cfg(test)]
mod tests {
    use crate::test_utils::{common_test_debug, common_test_display};

    #[test]
    fn test_bool_display() {
        common_test_display(true);
    }

    #[test]
    fn test_bool_debug() {
        common_test_debug(true);
    }

    #[test]
    fn test_f32_display() {
        common_test_display(123.4f32);
    }

    #[test]
    fn test_f32_debug() {
        common_test_debug(123.4f32);
    }

    #[test]
    fn test_f64_display() {
        common_test_display(123.4f64);
    }

    #[test]
    fn test_f64_debug() {
        common_test_debug(123.4f64);
    }

    #[test]
    fn test_i8_display() {
        common_test_display(-123i8);
    }

    #[test]
    fn test_i8_debug() {
        common_test_debug(-123i8);
    }

    #[test]
    fn test_i16_display() {
        common_test_display(-1234i16);
    }

    #[test]
    fn test_i16_debug() {
        common_test_debug(-1234i16);
    }

    #[test]
    fn test_i32_display() {
        common_test_display(-123456i32);
    }

    #[test]
    fn test_i32_debug() {
        common_test_debug(-123456i32);
    }

    #[test]
    fn test_i64_display() {
        common_test_display(-1200000000000000000i64);
    }

    #[test]
    fn test_i64_debug() {
        common_test_debug(-1200000000000000000i64);
    }

    #[test]
    fn test_u8_display() {
        common_test_display(123u8);
    }

    #[test]
    fn test_u8_debug() {
        common_test_debug(123u8);
    }

    #[test]
    fn test_u16_display() {
        common_test_display(1234u16);
    }

    #[test]
    fn test_u16_debug() {
        common_test_debug(1234u16);
    }

    #[test]
    fn test_u32_display() {
        common_test_display(123456u32);
    }

    #[test]
    fn test_u32_debug() {
        common_test_debug(123456u32);
    }

    #[test]
    fn test_u64_display() {
        common_test_display(1200000000000000000u64);
    }

    #[test]
    fn test_u64_debug() {
        common_test_debug(1200000000000000000u64);
    }

    #[test]
    fn test_str_display() {
        common_test_display("test");
    }

    #[test]
    fn test_str_debug() {
        common_test_debug("test");
    }

    #[test]
    fn test_isize_display() {
        common_test_display(-1200000000000000000isize);
    }

    #[test]
    fn test_isize_debug() {
        common_test_debug(-1200000000000000000isize);
    }

    #[test]
    fn test_usize_display() {
        common_test_display(1200000000000000000usize);
    }

    #[test]
    fn test_usize_debug() {
        common_test_debug(1200000000000000000usize);
    }
}
