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

//! Tests for `ScoreDebug` derive macro.
//!
//! Only positive paths can be checked with regular unit tests.
//! This is due to error paths resulting in compilation errors (as expected with proc macros).
//!
//! Results are compared with Rust built-in `Debug` derive macro.

// TODO: tests fail for `+nightly-2025-05-30` and older.
// Remove `#[cfg(not(miri))]` once updated in CI.
// https://github.com/eclipse-score/baselibs_rust/issues/31

mod utils;

use crate::utils::StringWriter;
use mw_log_fmt::{write, ScoreDebug};
use mw_log_fmt_macro::{mw_log_format_args, ScoreDebug};

#[test]
#[cfg(not(miri))]
fn test_struct_named() {
    #[derive(Debug, ScoreDebug)]
    struct Point {
        x: i32,
        y: i32,
        name: String,
    }

    let p = Point {
        x: 123,
        y: -321,
        name: "example".to_string(),
    };

    let args = mw_log_format_args!("{:?}", p);
    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));

    // Compare with Rust built-in `Debug` derive macro.
    let expected = format!("{:?}", p);
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_struct_unnamed() {
    #[derive(Debug, ScoreDebug)]
    struct Point(i32, i32, String);

    let p = Point(123, -123, "example".to_string());

    let args = mw_log_format_args!("{:?}", p);
    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));

    // Compare with Rust built-in `Debug` derive macro.
    let expected = format!("{:?}", p);
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_struct_unit() {
    #[derive(Debug, ScoreDebug)]
    struct UnitStruct;

    let unit_struct = UnitStruct;

    let args = mw_log_format_args!("{:?}", unit_struct);
    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));

    // Compare with Rust built-in `Debug` derive macro.
    let expected = format!("{:?}", unit_struct);
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_struct_generics() {
    #[derive(Debug, ScoreDebug)]
    // #[derive(Debug)]
    struct Example<'a, const N: usize, T: PartialEq + ScoreDebug> {
        lifetime: &'a str,
        generic: [T; N],
    }

    let p = Example {
        lifetime: "example",
        generic: [123; 10],
    };

    let args = mw_log_format_args!("{:?}", p);
    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));

    // Compare with Rust built-in `Debug` derive macro.
    let expected = format!("{:?}", p);
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_enum() {
    #[allow(dead_code)]
    #[derive(Debug, ScoreDebug)]
    enum Flag {
        Ignored,
        Optional,
        Required,
    }

    let flag = Flag::Optional;

    let args = mw_log_format_args!("{:?}", flag);
    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));

    // Compare with Rust built-in `Debug` derive macro.
    let expected = format!("{:?}", flag);
    assert_eq!(w.get(), expected);
}
