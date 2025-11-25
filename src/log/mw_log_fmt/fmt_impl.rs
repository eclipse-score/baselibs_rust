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

//! `ScoreDebug` implementations for common types.

use crate::builders::DebugList;
use crate::fmt::{Error, Result, ScoreDebug, Writer};
use crate::fmt_spec::FormatSpec;

macro_rules! impl_debug_for_t {
    ($t:ty, $fn:ident) => {
        impl ScoreDebug for $t {
            fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
                f.$fn(self, spec)
            }
        }
    };
}

impl_debug_for_t!(bool, write_bool);
impl_debug_for_t!(f32, write_f32);
impl_debug_for_t!(f64, write_f64);
impl_debug_for_t!(i8, write_i8);
impl_debug_for_t!(i16, write_i16);
impl_debug_for_t!(i32, write_i32);
impl_debug_for_t!(i64, write_i64);
impl_debug_for_t!(u8, write_u8);
impl_debug_for_t!(u16, write_u16);
impl_debug_for_t!(u32, write_u32);
impl_debug_for_t!(u64, write_u64);

impl ScoreDebug for str {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        let empty_spec = FormatSpec::new();
        f.write_str("\"", &empty_spec)?;
        f.write_str(self, spec)?;
        f.write_str("\"", &empty_spec)
    }
}

impl ScoreDebug for String {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        ScoreDebug::fmt(&self.as_str(), f, spec)
    }
}

macro_rules! impl_debug_for_t_casted {
    ($ti:ty, $to:ty, $fn:ident) => {
        impl ScoreDebug for $ti {
            fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
                let casted = <$to>::try_from(*self).map_err(|_| Error)?;
                f.$fn(&casted, spec)
            }
        }
    };
}

#[cfg(target_pointer_width = "32")]
impl_debug_for_t_casted!(isize, i32, write_i32);
#[cfg(target_pointer_width = "64")]
impl_debug_for_t_casted!(isize, i64, write_i64);
#[cfg(target_pointer_width = "32")]
impl_debug_for_t_casted!(usize, u32, write_u32);
#[cfg(target_pointer_width = "64")]
impl_debug_for_t_casted!(usize, u64, write_u64);

impl<T: ScoreDebug + ?Sized> ScoreDebug for &T {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        ScoreDebug::fmt(&**self, f, spec)
    }
}

impl<T: ScoreDebug + ?Sized> ScoreDebug for &mut T {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        ScoreDebug::fmt(&**self, f, spec)
    }
}

impl<T: ScoreDebug> ScoreDebug for [T] {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        let mut debug_list = DebugList::new(f, spec);
        debug_list.entries(self.iter()).finish()
    }
}

impl<T: ScoreDebug, const N: usize> ScoreDebug for [T; N] {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        ScoreDebug::fmt(&&self[..], f, spec)
    }
}

impl<T: ScoreDebug> ScoreDebug for Vec<T> {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        ScoreDebug::fmt(&**self, f, spec)
    }
}

impl<T: ScoreDebug> ScoreDebug for std::rc::Rc<T> {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        ScoreDebug::fmt(&**self, f, spec)
    }
}

impl<T: ScoreDebug> ScoreDebug for std::sync::Arc<T> {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        ScoreDebug::fmt(&**self, f, spec)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::common_test_debug;

    #[test]
    fn test_bool_debug() {
        common_test_debug(true);
    }

    #[test]
    fn test_f32_debug() {
        common_test_debug(123.4f32);
    }

    #[test]
    fn test_f64_debug() {
        common_test_debug(123.4f64);
    }

    #[test]
    fn test_i8_debug() {
        common_test_debug(-123i8);
    }

    #[test]
    fn test_i16_debug() {
        common_test_debug(-1234i16);
    }

    #[test]
    fn test_i32_debug() {
        common_test_debug(-123456i32);
    }

    #[test]
    fn test_i64_debug() {
        common_test_debug(-1200000000000000000i64);
    }

    #[test]
    fn test_u8_debug() {
        common_test_debug(123u8);
    }

    #[test]
    fn test_u16_debug() {
        common_test_debug(1234u16);
    }

    #[test]
    fn test_u32_debug() {
        common_test_debug(123456u32);
    }

    #[test]
    fn test_u64_debug() {
        common_test_debug(1200000000000000000u64);
    }

    #[test]
    fn test_str_debug() {
        common_test_debug("test");
    }

    #[test]
    fn test_string_debug() {
        common_test_debug(String::from("test"));
    }

    #[test]
    fn test_isize_debug() {
        common_test_debug(-1200000000000000000isize);
    }

    #[test]
    fn test_usize_debug() {
        common_test_debug(1200000000000000000usize);
    }

    #[test]
    fn test_slice_debug() {
        common_test_debug([123, 456, 789].as_slice());
    }

    #[test]
    fn test_array_debug() {
        common_test_debug([123, 456, 789]);
    }

    #[test]
    fn test_vec_debug() {
        common_test_debug(vec![987, 654, 321, 159]);
    }

    #[test]
    fn test_rc_debug() {
        let rc = std::rc::Rc::new(444);
        common_test_debug(rc);
    }

    #[test]
    fn test_arc_debug() {
        let arc = std::sync::Arc::new(654);
        common_test_debug(arc);
    }
}
