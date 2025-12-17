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

//! String-based Rust backend for `mw_log`.
//! Data is written to a fixed-size buffer.

use core::fmt::Write;
use mw_log::fmt::{score_write, write, Error, FormatSpec, Result, ScoreWrite};
use mw_log::{LevelFilter, Log, Metadata, Record};

/// Fixed size buffer for strings.
struct FixedBuf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> FixedBuf<N> {
    pub const fn new() -> Self {
        Self { buf: [0; N], len: 0 }
    }

    /// Get buffer as a string.
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }

    /// Get number of remaining bytes in the buffer.
    pub fn remaining(&self) -> usize {
        N - self.len
    }
}

impl<const N: usize> Default for FixedBuf<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Write for FixedBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Get number of remaining bytes in the buffer.
        // Return if buffer is full.
        let remaining = self.remaining();
        if remaining == 0 {
            return Ok(());
        }

        // Get provided string as bytes.
        let bytes = s.as_bytes();

        // Get number of bytes requested or remaining in the buffer.
        let mut end = bytes.len().min(remaining);

        // Move back until char boundary.
        // Return if buffer is full.
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        if end == 0 {
            return Ok(());
        }

        // Write to underlying buffer.
        self.buf[self.len..self.len + end].copy_from_slice(&bytes[..end]);
        self.len += end;

        Ok(())
    }
}

/// Writer implementation based on fixed size buffer.
#[derive(Default)]
struct FixedBufWriter<const N: usize> {
    buf: FixedBuf<N>,
}

impl<const N: usize> FixedBufWriter<N> {
    /// Create `FixedBufWriter` instance.
    pub fn new() -> Self {
        Self { buf: FixedBuf::new() }
    }

    /// Get data from buffer.
    pub fn get(&self) -> &str {
        self.buf.as_str()
    }
}

impl<const N: usize> ScoreWrite for FixedBufWriter<N> {
    fn write_bool(&mut self, v: &bool, _spec: &FormatSpec) -> Result {
        write!(self.buf, "{}", v).map_err(|_| Error)
    }

    fn write_f32(&mut self, v: &f32, _spec: &FormatSpec) -> Result {
        write!(self.buf, "{}", v).map_err(|_| Error)
    }

    fn write_f64(&mut self, v: &f64, _spec: &FormatSpec) -> Result {
        write!(self.buf, "{}", v).map_err(|_| Error)
    }

    fn write_i8(&mut self, v: &i8, _spec: &FormatSpec) -> Result {
        write!(self.buf, "{}", v).map_err(|_| Error)
    }

    fn write_i16(&mut self, v: &i16, _spec: &FormatSpec) -> Result {
        write!(self.buf, "{}", v).map_err(|_| Error)
    }

    fn write_i32(&mut self, v: &i32, _spec: &FormatSpec) -> Result {
        write!(self.buf, "{}", v).map_err(|_| Error)
    }

    fn write_i64(&mut self, v: &i64, _spec: &FormatSpec) -> Result {
        write!(self.buf, "{}", v).map_err(|_| Error)
    }

    fn write_u8(&mut self, v: &u8, _spec: &FormatSpec) -> Result {
        write!(self.buf, "{}", v).map_err(|_| Error)
    }

    fn write_u16(&mut self, v: &u16, _spec: &FormatSpec) -> Result {
        write!(self.buf, "{}", v).map_err(|_| Error)
    }

    fn write_u32(&mut self, v: &u32, _spec: &FormatSpec) -> Result {
        write!(self.buf, "{}", v).map_err(|_| Error)
    }

    fn write_u64(&mut self, v: &u64, _spec: &FormatSpec) -> Result {
        write!(self.buf, "{}", v).map_err(|_| Error)
    }

    fn write_str(&mut self, v: &str, _spec: &FormatSpec) -> Result {
        write!(self.buf, "{}", v).map_err(|_| Error)
    }
}

/// Builder for the `MwLogger`.
pub struct MwLoggerBuilder {
    context: String,
    show_module: bool,
    show_file: bool,
    show_line: bool,
    log_level: LevelFilter,
}

impl MwLoggerBuilder {
    /// Create builder with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set context for the `MwLogger`.
    pub fn context(mut self, context: &str) -> Self {
        self.context = context.to_string();
        self
    }

    /// Show module name in logs.
    pub fn show_module(mut self, show_module: bool) -> Self {
        self.show_module = show_module;
        self
    }

    /// Show file name in logs.
    pub fn show_file(mut self, show_file: bool) -> Self {
        self.show_file = show_file;
        self
    }

    /// Show line number in logs.
    pub fn show_line(mut self, show_line: bool) -> Self {
        self.show_line = show_line;
        self
    }

    /// Filter logs by level.
    pub fn log_level(mut self, log_level: LevelFilter) -> Self {
        self.log_level = log_level;
        self
    }

    /// Build the `MwLogger` with provided context and configuration.
    pub fn build(self) -> MwLogger {
        MwLogger {
            context: self.context,
            show_module: self.show_module,
            show_file: self.show_file,
            show_line: self.show_line,
            log_level: self.log_level,
        }
    }

    /// Build the `MwLogger` and set it as the default logger.
    pub fn set_as_default_logger(self) {
        let logger = self.build();
        mw_log::set_max_level(logger.log_level());
        if mw_log::set_logger(Box::new(logger)).is_err() {
            panic!("unable to set logger");
        }
    }
}

impl Default for MwLoggerBuilder {
    fn default() -> Self {
        Self {
            context: "DFLT".to_string(),
            show_module: true,
            show_file: true,
            show_line: true,
            log_level: LevelFilter::Off,
        }
    }
}

/// String-based logger implementation.
pub struct MwLogger {
    context: String,
    show_module: bool,
    show_file: bool,
    show_line: bool,
    log_level: LevelFilter,
}

impl MwLogger {
    /// Current log level.
    pub fn log_level(&self) -> LevelFilter {
        self.log_level
    }
}

impl Log for MwLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.log_level()
    }

    fn context(&self) -> &str {
        &self.context
    }

    fn log(&self, record: &Record) {
        // Finish early if not enabled for requested level.
        let metadata = record.metadata();
        if !self.enabled(metadata) {
            return;
        }

        // Create writer.
        let mut writer = FixedBufWriter::<512>::new();

        // Write module, file and line.
        if self.show_module || self.show_file || self.show_line {
            let _ = score_write!(&mut writer, "[");
            if self.show_module {
                let _ = score_write!(&mut writer, "{}:", record.module_path());
            }
            if self.show_file {
                let _ = score_write!(&mut writer, "{}:", record.file());
            }
            if self.show_line {
                let _ = score_write!(&mut writer, "{}", record.line());
            }
            let _ = score_write!(&mut writer, "]");
        }

        // Write context and log level.
        let context = record.context();
        let level = metadata.level().as_str();
        let _ = score_write!(&mut writer, "[{}][{}] ", context, level);

        // Write log data.
        let _ = write(&mut writer, *record.args());

        // Print to stderr.
        eprintln!("{}", writer.get());
    }

    fn flush(&self) {
        // No-op.
    }
}
