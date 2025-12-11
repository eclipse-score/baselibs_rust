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

//! WARNING: this is not part of the crate's public API and is subject to change at any time.

use crate::{Level, Log, Metadata, Record};
use core::panic::Location;
use mw_log_fmt::Arguments;
// Following imports are necessary for macros.
pub use mw_log_fmt_macro::mw_log_format_args as format_args;
pub use mw_log_fmt_macro::mw_log_format_args_nl as format_args_nl;

pub fn log<L: Log>(
    logger: L,
    args: Arguments<'_>,
    level: Level,
    &(context, module_path, loc): &(&str, &'static str, &'static Location),
) {
    let mut builder = Record::builder();

    builder
        .args(args)
        .level(level)
        .context(context)
        .module_path(Some(module_path))
        .file(Some(loc.file()))
        .line(Some(loc.line()));

    logger.log(&builder.build());
}

pub fn enabled<L: Log>(logger: L, level: Level, context: &str) -> bool {
    logger.enabled(&Metadata::builder().level(level).context(context).build())
}

#[track_caller]
pub fn loc() -> &'static Location<'static> {
    Location::caller()
}
