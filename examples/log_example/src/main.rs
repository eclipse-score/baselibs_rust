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

//! Example app containing basic logger implementation.

mod logger;

use crate::logger::ExampleLogger;
use mw_log::{debug, error, fatal, info, trace, warn, LevelFilter};

fn main() {
    // Initialize logger.
    mw_log::set_max_level(LevelFilter::Info);
    let result = mw_log::set_global_logger(Box::new(ExampleLogger));
    if result.is_err() {
        panic!("unable to set logger")
    }

    // Example logs.
    trace!("trace log - hidden!");
    debug!("debug log - hidden!");
    info!("info log");
    warn!("warn log");
    error!("error log");
    fatal!("fatal log");
}
