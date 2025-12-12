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

//! Replacement for macros provided by Rust compiler:
//! - [`mw_log_format_args!`] - replacement for `format_args!`
//! - [`ScoreDebug`] - replacement for `Debug`

// All errors should result in compilation error.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

mod format_args;
mod score_debug;

/// Constructs parameters for the other string-formatting macros.
///
/// This macro takes a formatting string literal containing `{}` for each additional argument.
/// [`mw_log_format_args!`] prepares the additional parameters to ensure the output can be interpreted as a message.
#[proc_macro]
pub fn mw_log_format_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    format_args::expand(input)
}

/// Constructs parameters for the other string-formatting macros, with newline appended at the end.
///
/// See [`mw_log_format_args!`] for more information.
#[proc_macro]
pub fn mw_log_format_args_nl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    format_args::expand_nl(input)
}

/// Automatically generate [`ScoreDebug`] implementation.
#[proc_macro_derive(ScoreDebug)]
pub fn score_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    score_debug::expand(input)
}
