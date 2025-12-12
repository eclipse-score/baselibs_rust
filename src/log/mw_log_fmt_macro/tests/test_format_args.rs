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

//! Tests for `mw_log_format_args` and `mw_log_format_args_nl`.
//!
//! Only positive paths can be checked with regular unit tests.
//! This is due to error paths resulting in compilation errors (as expected with proc macros).
//!
//! Results are compared with Rust built-in `format_args` macro.

// TODO: tests fail for `+nightly-2025-05-30` and older.
// Remove `#[cfg(not(miri))]` once updated in CI.
// https://github.com/eclipse-score/baselibs_rust/issues/31

mod utils;

use crate::utils::StringWriter;
use mw_log_fmt::{write, Alignment, DebugAsHex, DisplayHint, Fragment, Sign};
use mw_log_fmt_macro::{mw_log_format_args, mw_log_format_args_nl};

#[test]
fn test_single_literal() {
    let args = mw_log_format_args!("test_string");

    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));
    assert_eq!(args.0.len(), 1);
    assert_eq!(w.get(), "test_string");

    // Compare with Rust built-in format args.
    #[allow(clippy::useless_format)]
    let expected = format!("test_string");
    assert_eq!(w.get(), expected);
}

#[test]
fn test_escaped_braces() {
    let args = mw_log_format_args!("{{}}}}{{");

    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));
    assert_eq!(args.0.len(), 1);
    assert_eq!(w.get(), "{}}{");

    // Compare with Rust built-in format args.
    #[allow(clippy::useless_format)]
    let expected = format!("{{}}}}{{");
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_single_placeholder() {
    let args = mw_log_format_args!("{}", 123);

    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));
    assert_eq!(args.0.len(), 1);
    assert_eq!(w.get(), "123");

    // Compare with Rust built-in format args.
    let expected = format!("{}", 123);
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_mixed_literals_and_placeholders() {
    let args = mw_log_format_args!("test_{}_string", 321);

    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));
    assert_eq!(args.0.len(), 3);
    assert_eq!(w.get(), "test_321_string");

    // Compare with Rust built-in format args.
    let expected = format!("test_{}_string", 321);
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_arg_index() {
    let args = mw_log_format_args!("test_{2}_{1}_{0}", 123, 234, 345);

    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));
    assert_eq!(args.0.len(), 6);
    assert_eq!(w.get(), "test_345_234_123");

    // Compare with Rust built-in format args.
    let expected = format!("test_{2}_{1}_{0}", 123, 234, 345);
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_arg_pos_and_index() {
    let args = mw_log_format_args!("test_{2}_{}_{1}_{}_{0}", 123, 234, 345);

    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));
    assert_eq!(args.0.len(), 10);
    assert_eq!(w.get(), "test_345_123_234_234_123");

    // Compare with Rust built-in format args.
    let expected = format!("test_{2}_{}_{1}_{}_{0}", 123, 234, 345);
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_arg_name() {
    let x1 = 123;
    let x2 = 234;
    let x3 = 345;
    let args = mw_log_format_args!("test_{x3}_{x2}_{x1}", x1, x2, x3);

    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));
    assert_eq!(args.0.len(), 6);
    assert_eq!(w.get(), "test_345_234_123");

    // Compare with Rust built-in format args.
    // NOTE: known misalignment.
    // It is not allowed to have redundant arguments in Rust (`("{x1}", x1)`).
    // This is currently not possible to do using `mw_log_format_args`.
    let expected = format!("test_{x3}_{x2}_{x1}");
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_arg_name_alias() {
    let x1 = 123;
    let x2 = 234;
    let x3 = 345;
    let args = mw_log_format_args!("test_{a3}_{a2}_{a1}", a1 = x1, a2 = x2, a3 = x3);

    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));
    assert_eq!(args.0.len(), 6);
    assert_eq!(w.get(), "test_345_234_123");

    // Compare with Rust built-in format args.
    let expected = format!("test_{a3}_{a2}_{a1}", a1 = x1, a2 = x2, a3 = x3);
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_arg_pos_and_name() {
    let x1 = 123;
    let x2 = 234;
    let x3 = 345;
    let args = mw_log_format_args!("test_{x3}_{}_{x2}_{}_{x1}", x1, x2, x3);

    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));
    assert_eq!(args.0.len(), 10);
    assert_eq!(w.get(), "test_345_123_234_234_123");

    // Compare with Rust built-in format args.
    // NOTE: known misalignment.
    // It is not allowed to have redundant arguments in Rust (`("{x1}", x1)`).
    // This is currently not possible to do using `mw_log_format_args`.
    let expected = format!("test_{x3}_{}_{x2}_{}_{x1}", x1, x2);
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_arg_mixed() {
    let x1 = 111;
    let x2 = 222;
    let args = mw_log_format_args!("test_{x1}_{1}_{}", x1, x2);

    let mut w = StringWriter::new();
    let _ = write(&mut w, args).map_err(|_| panic!("write failed"));
    assert_eq!(args.0.len(), 6);
    assert_eq!(w.get(), "test_111_222_111");

    // Compare with Rust built-in format args.
    let expected = format!("test_{x1}_{1}_{}", x1, x2);
    assert_eq!(w.get(), expected);
}

#[test]
#[cfg(not(miri))]
fn test_format_spec_empty() {
    let args = mw_log_format_args!("{:}", 123);

    let placeholder = match args.0.first().unwrap() {
        Fragment::Literal(_) => panic!("invalid variant"),
        Fragment::Placeholder(placeholder) => placeholder,
    };

    let format_spec = placeholder.format_spec();
    assert!(format_spec.get_display_hint() == DisplayHint::NoHint);
    assert_eq!(format_spec.get_fill(), ' ');
    assert!(format_spec.get_align().is_none());
    assert!(format_spec.get_sign().is_none());
    assert!(!format_spec.get_alternate());
    assert!(!format_spec.get_zero_pad());
    assert!(format_spec.get_debug_as_hex().is_none());
    assert_eq!(format_spec.get_width(), None);
    assert_eq!(format_spec.get_precision(), None);
}

#[test]
#[cfg(not(miri))]
fn test_format_spec_all() {
    let args = mw_log_format_args!("{:c<-#0333.555x}", 123);

    let placeholder = match args.0.first().unwrap() {
        Fragment::Literal(_) => panic!("invalid variant"),
        Fragment::Placeholder(placeholder) => placeholder,
    };

    let format_spec = placeholder.format_spec();
    assert!(format_spec.get_display_hint() == DisplayHint::LowerHex);
    assert_eq!(format_spec.get_fill(), 'c');
    assert!(format_spec.get_align() == Some(Alignment::Left));
    assert!(format_spec.get_sign() == Some(Sign::Minus));
    assert!(format_spec.get_alternate());
    assert!(format_spec.get_zero_pad());
    assert!(format_spec.get_debug_as_hex().is_none());
    assert_eq!(format_spec.get_width(), Some(333));
    assert_eq!(format_spec.get_precision(), Some(555));
}

#[test]
#[cfg(not(miri))]
fn test_format_spec_debug() {
    let args = mw_log_format_args!("{:#X?}", 123);

    let placeholder = match args.0.first().unwrap() {
        Fragment::Literal(_) => panic!("invalid variant"),
        Fragment::Placeholder(placeholder) => placeholder,
    };

    let format_spec = placeholder.format_spec();
    assert!(format_spec.get_display_hint() == DisplayHint::Debug);
    assert_eq!(format_spec.get_fill(), ' ');
    assert!(format_spec.get_align().is_none());
    assert!(format_spec.get_sign().is_none());
    assert!(format_spec.get_alternate());
    assert!(!format_spec.get_zero_pad());
    assert!(format_spec.get_debug_as_hex() == Some(DebugAsHex::Upper));
    assert_eq!(format_spec.get_width(), None);
    assert_eq!(format_spec.get_precision(), None);
}

#[test]
#[cfg(not(miri))]
fn test_format_spec_display_hint_octal() {
    let args = mw_log_format_args!("{:o}", 123);

    let placeholder = match args.0.first().unwrap() {
        Fragment::Literal(_) => panic!("invalid variant"),
        Fragment::Placeholder(placeholder) => placeholder,
    };

    let format_spec = placeholder.format_spec();
    assert!(format_spec.get_display_hint() == DisplayHint::Octal);
}

#[test]
#[cfg(not(miri))]
fn test_format_spec_display_hint_lower_hex() {
    let args = mw_log_format_args!("{:x}", 123);

    let placeholder = match args.0.first().unwrap() {
        Fragment::Literal(_) => panic!("invalid variant"),
        Fragment::Placeholder(placeholder) => placeholder,
    };

    let format_spec = placeholder.format_spec();
    assert!(format_spec.get_display_hint() == DisplayHint::LowerHex);
}

#[test]
#[cfg(not(miri))]
fn test_format_spec_display_hint_upper_hex() {
    let args = mw_log_format_args!("{:X}", 123);

    let placeholder = match args.0.first().unwrap() {
        Fragment::Literal(_) => panic!("invalid variant"),
        Fragment::Placeholder(placeholder) => placeholder,
    };

    let format_spec = placeholder.format_spec();
    assert!(format_spec.get_display_hint() == DisplayHint::UpperHex);
}

#[test]
#[cfg(not(miri))]
fn test_format_spec_display_hint_pointer() {
    let args = mw_log_format_args!("{:p}", 123);

    let placeholder = match args.0.first().unwrap() {
        Fragment::Literal(_) => panic!("invalid variant"),
        Fragment::Placeholder(placeholder) => placeholder,
    };

    let format_spec = placeholder.format_spec();
    assert!(format_spec.get_display_hint() == DisplayHint::Pointer);
}

#[test]
#[cfg(not(miri))]
fn test_format_spec_display_hint_binary() {
    let args = mw_log_format_args!("{:b}", 123);

    let placeholder = match args.0.first().unwrap() {
        Fragment::Literal(_) => panic!("invalid variant"),
        Fragment::Placeholder(placeholder) => placeholder,
    };

    let format_spec = placeholder.format_spec();
    assert!(format_spec.get_display_hint() == DisplayHint::Binary);
}

#[test]
#[cfg(not(miri))]
fn test_format_spec_display_hint_lower_exp() {
    let args = mw_log_format_args!("{:e}", 123);

    let placeholder = match args.0.first().unwrap() {
        Fragment::Literal(_) => panic!("invalid variant"),
        Fragment::Placeholder(placeholder) => placeholder,
    };

    let format_spec = placeholder.format_spec();
    assert!(format_spec.get_display_hint() == DisplayHint::LowerExp);
}

#[test]
#[cfg(not(miri))]
fn test_format_spec_display_hint_upper_exp() {
    let args = mw_log_format_args!("{:E}", 123);

    let placeholder = match args.0.first().unwrap() {
        Fragment::Literal(_) => panic!("invalid variant"),
        Fragment::Placeholder(placeholder) => placeholder,
    };

    let format_spec = placeholder.format_spec();
    assert!(format_spec.get_display_hint() == DisplayHint::UpperExp);
}

#[test]
fn test_format_args_nl() {
    let args = mw_log_format_args_nl!("test_string");
    let args = args.0;

    assert_eq!(args.len(), 2);
    // Check literal string.
    let f1 = args.first().unwrap();
    match f1 {
        Fragment::Literal(s) => assert_eq!(*s, "test_string"),
        Fragment::Placeholder(_placeholder) => panic!("invalid variant"),
    }
    // Check newline.
    let f2 = args.get(1).unwrap();
    match f2 {
        Fragment::Literal(s) => assert_eq!(*s, "\n"),
        Fragment::Placeholder(_placeholder) => panic!("invalid variant"),
    }
}
