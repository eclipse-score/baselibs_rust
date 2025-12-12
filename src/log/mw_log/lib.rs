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

//! A lightweight logging facade.

#![warn(missing_docs)]
#![deny(unconditional_recursion)]

#[cfg(any(
    all(feature = "max_level_off", feature = "max_level_fatal"),
    all(feature = "max_level_off", feature = "max_level_error"),
    all(feature = "max_level_off", feature = "max_level_warn"),
    all(feature = "max_level_off", feature = "max_level_info"),
    all(feature = "max_level_off", feature = "max_level_debug"),
    all(feature = "max_level_off", feature = "max_level_trace"),
    all(feature = "max_level_fatal", feature = "max_level_error"),
    all(feature = "max_level_fatal", feature = "max_level_warn"),
    all(feature = "max_level_fatal", feature = "max_level_info"),
    all(feature = "max_level_fatal", feature = "max_level_debug"),
    all(feature = "max_level_fatal", feature = "max_level_trace"),
    all(feature = "max_level_error", feature = "max_level_warn"),
    all(feature = "max_level_error", feature = "max_level_info"),
    all(feature = "max_level_error", feature = "max_level_debug"),
    all(feature = "max_level_error", feature = "max_level_trace"),
    all(feature = "max_level_warn", feature = "max_level_info"),
    all(feature = "max_level_warn", feature = "max_level_debug"),
    all(feature = "max_level_warn", feature = "max_level_trace"),
    all(feature = "max_level_info", feature = "max_level_debug"),
    all(feature = "max_level_info", feature = "max_level_trace"),
    all(feature = "max_level_debug", feature = "max_level_trace"),
))]
compile_error!("multiple max_level_* features set");

#[rustfmt::skip]
#[cfg(any(
    all(feature = "release_max_level_off", feature = "release_max_level_fatal"),
    all(feature = "release_max_level_off", feature = "release_max_level_error"),
    all(feature = "release_max_level_off", feature = "release_max_level_warn"),
    all(feature = "release_max_level_off", feature = "release_max_level_info"),
    all(feature = "release_max_level_off", feature = "release_max_level_debug"),
    all(feature = "release_max_level_off", feature = "release_max_level_trace"),
    all(feature = "release_max_level_fatal", feature = "release_max_level_error"),
    all(feature = "release_max_level_fatal", feature = "release_max_level_warn"),
    all(feature = "release_max_level_fatal", feature = "release_max_level_info"),
    all(feature = "release_max_level_fatal", feature = "release_max_level_debug"),
    all(feature = "release_max_level_fatal", feature = "release_max_level_trace"),
    all(feature = "release_max_level_error", feature = "release_max_level_warn"),
    all(feature = "release_max_level_error", feature = "release_max_level_info"),
    all(feature = "release_max_level_error", feature = "release_max_level_debug"),
    all(feature = "release_max_level_error", feature = "release_max_level_trace"),
    all(feature = "release_max_level_warn", feature = "release_max_level_info"),
    all(feature = "release_max_level_warn", feature = "release_max_level_debug"),
    all(feature = "release_max_level_warn", feature = "release_max_level_trace"),
    all(feature = "release_max_level_info", feature = "release_max_level_debug"),
    all(feature = "release_max_level_info", feature = "release_max_level_trace"),
    all(feature = "release_max_level_debug", feature = "release_max_level_trace"),
))]
compile_error!("multiple release_max_level_* features set");

use core::str::FromStr;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::{cmp, mem};
pub use mw_log_fmt as fmt;
use mw_log_fmt::Arguments;
use mw_log_fmt_macro::mw_log_format_args;
pub use mw_log_fmt_macro::ScoreDebug;

#[macro_use]
mod macros;

/// Pointer to the global logger.
static mut LOGGER: &dyn Log = &NopLogger;

/// State of [`LOGGER`] initialization.
static STATE: AtomicUsize = AtomicUsize::new(0);

/// [`LOGGER`] not initialized.
const UNINITIALIZED: usize = 0;
/// [`LOGGER`] during initialization.
const INITIALIZING: usize = 1;
/// [`LOGGER`] initialized.
const INITIALIZED: usize = 2;

static MAX_LOG_LEVEL_FILTER: AtomicUsize = AtomicUsize::new(0);

static LOG_LEVEL_NAMES: [&str; 7] = ["OFF", "FATAL", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

static SET_LOGGER_ERROR: &str = "attempted to set a logger after the logging system \
                                 was already initialized";
static LEVEL_PARSE_ERROR: &str =
    "attempted to convert a string that doesn't match an existing log level";

/// An enum representing the available verbosity levels of the logger.
#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Level {
    /// Highest level, for extremely serious errors.
    Fatal = 1,
    /// Designates very serious errors.
    Error,
    /// Designates hazardous situations.
    Warn,
    /// Designates useful information.
    Info,
    /// Designates lower priority information.
    Debug,
    /// Designates very low priority, often extremely verbose, information.
    Trace,
}

impl PartialEq<LevelFilter> for Level {
    #[inline]
    fn eq(&self, other: &LevelFilter) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialOrd<LevelFilter> for Level {
    #[inline]
    fn partial_cmp(&self, other: &LevelFilter) -> Option<cmp::Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }
}

impl FromStr for Level {
    type Err = ParseLevelError;
    fn from_str(level: &str) -> Result<Level, Self::Err> {
        LOG_LEVEL_NAMES
            .iter()
            .position(|&name| name.eq_ignore_ascii_case(level))
            .into_iter()
            .filter(|&idx| idx != 0)
            .map(|idx| Level::from_usize(idx).unwrap())
            .next()
            .ok_or(ParseLevelError(()))
    }
}

impl core::fmt::Display for Level {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.pad(self.as_str())
    }
}

impl Level {
    fn from_usize(u: usize) -> Option<Level> {
        match u {
            1 => Some(Level::Fatal),
            2 => Some(Level::Error),
            3 => Some(Level::Warn),
            4 => Some(Level::Info),
            5 => Some(Level::Debug),
            6 => Some(Level::Trace),
            _ => None,
        }
    }

    /// Returns the most verbose logging level.
    #[inline]
    pub fn max() -> Level {
        Level::Trace
    }

    /// Converts the [`Level`] to the equivalent [`LevelFilter`].
    #[inline]
    pub fn to_level_filter(&self) -> LevelFilter {
        LevelFilter::from_usize(*self as usize).unwrap()
    }

    /// Returns the string representation of the [`Level`].
    ///
    /// This returns the same string as the [`fmt::Display`] implementation.
    pub fn as_str(&self) -> &'static str {
        LOG_LEVEL_NAMES[*self as usize]
    }

    /// Iterate through all supported logging levels.
    ///
    /// The order of iteration is from more severe to less severe log messages.
    ///
    /// # Examples
    ///
    /// ```
    /// use mw_log::Level;
    ///
    /// let mut levels = Level::iter();
    ///
    /// assert_eq!(Some(Level::Fatal), levels.next());
    /// assert_eq!(Some(Level::Trace), levels.last());
    /// ```
    pub fn iter() -> impl Iterator<Item = Self> {
        (1..7).map(|i| Self::from_usize(i).unwrap())
    }

    /// Get the next-highest [`Level`] from this one.
    ///
    /// If the current [`Level`] is at the highest level, the returned [`Level`] will be the same as the current one.
    ///
    /// # Examples
    ///
    /// ```
    /// use mw_log::Level;
    ///
    /// let level = Level::Info;
    ///
    /// assert_eq!(Level::Debug, level.increment_severity());
    /// assert_eq!(Level::Trace, level.increment_severity().increment_severity());
    /// assert_eq!(Level::Trace, level.increment_severity().increment_severity().increment_severity()); // max level
    /// ```
    pub fn increment_severity(&self) -> Self {
        let current = *self as usize;
        Self::from_usize(current + 1).unwrap_or(*self)
    }

    /// Get the next-lowest [`Level`] from this one.
    ///
    /// If the current [`Level`] is at the lowest level, the returned [`Level`] will be the same as the current one.
    ///
    /// # Examples
    ///
    /// ```
    /// use mw_log::Level;
    ///
    /// let level = Level::Info;
    ///
    /// assert_eq!(Level::Warn, level.decrement_severity());
    /// assert_eq!(Level::Error, level.decrement_severity().decrement_severity());
    /// assert_eq!(Level::Fatal, level.decrement_severity().decrement_severity().decrement_severity());
    /// assert_eq!(Level::Fatal, level.decrement_severity().decrement_severity().decrement_severity().decrement_severity()); // min level
    /// ```
    pub fn decrement_severity(&self) -> Self {
        let current = *self as usize;
        Self::from_usize(current.saturating_sub(1)).unwrap_or(*self)
    }
}

/// An enum representing the available verbosity level filters of the logger.
///
/// A [`LevelFilter`] may be compared directly to a [`Level`].
#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum LevelFilter {
    /// A level lower than all log levels.
    Off = 0,
    /// Corresponds to the [`Level::Fatal`] log level.
    Fatal,
    /// Corresponds to the [`Level::Error`] log level.
    Error,
    /// Corresponds to the [`Level::Warn`] log level.
    Warn,
    /// Corresponds to the [`Level::Info`] log level.
    Info,
    /// Corresponds to the [`Level::Debug`] log level.
    Debug,
    /// Corresponds to the [`Level::Trace`] log level.
    Trace,
}

impl PartialEq<Level> for LevelFilter {
    #[inline]
    fn eq(&self, other: &Level) -> bool {
        other.eq(self)
    }
}

impl PartialOrd<Level> for LevelFilter {
    #[inline]
    fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }
}

impl FromStr for LevelFilter {
    type Err = ParseLevelError;
    fn from_str(level: &str) -> Result<LevelFilter, Self::Err> {
        LOG_LEVEL_NAMES
            .iter()
            .position(|&name| name.eq_ignore_ascii_case(level))
            .map(|p| LevelFilter::from_usize(p).unwrap())
            .ok_or(ParseLevelError(()))
    }
}

impl core::fmt::Display for LevelFilter {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.pad(self.as_str())
    }
}

impl LevelFilter {
    fn from_usize(u: usize) -> Option<LevelFilter> {
        match u {
            0 => Some(LevelFilter::Off),
            1 => Some(LevelFilter::Fatal),
            2 => Some(LevelFilter::Error),
            3 => Some(LevelFilter::Warn),
            4 => Some(LevelFilter::Info),
            5 => Some(LevelFilter::Debug),
            6 => Some(LevelFilter::Trace),
            _ => None,
        }
    }

    /// Returns the most verbose logging level filter.
    #[inline]
    pub fn max() -> LevelFilter {
        LevelFilter::Trace
    }

    /// Converts [`LevelFilter`] to the equivalent [`Level`].
    ///
    /// Returns [`None`] if [`LevelFilter`] is [`LevelFilter::Off`].
    #[inline]
    pub fn to_level(&self) -> Option<Level> {
        Level::from_usize(*self as usize)
    }

    /// Returns the string representation of the [`LevelFilter`].
    ///
    /// This returns the same string as the [`fmt::Display`] implementation.
    pub fn as_str(&self) -> &'static str {
        LOG_LEVEL_NAMES[*self as usize]
    }

    /// Iterate through all supported filtering levels.
    ///
    /// The order of iteration is from less to more verbose filtering.
    ///
    /// # Examples
    ///
    /// ```
    /// use mw_log::LevelFilter;
    ///
    /// let mut levels = LevelFilter::iter();
    ///
    /// assert_eq!(Some(LevelFilter::Off), levels.next());
    /// assert_eq!(Some(LevelFilter::Trace), levels.last());
    /// ```
    pub fn iter() -> impl Iterator<Item = Self> {
        (0..7).map(|i| Self::from_usize(i).unwrap())
    }

    /// Get the next-highest [`LevelFilter`] from this one.
    ///
    /// If the current [`LevelFilter`] is at the highest level, the returned [`LevelFilter`] will be the same as the current one.
    ///
    /// # Examples
    ///
    /// ```
    /// use mw_log::LevelFilter;
    ///
    /// let level_filter = LevelFilter::Info;
    ///
    /// assert_eq!(LevelFilter::Debug, level_filter.increment_severity());
    /// assert_eq!(LevelFilter::Trace, level_filter.increment_severity().increment_severity());
    /// assert_eq!(LevelFilter::Trace, level_filter.increment_severity().increment_severity().increment_severity()); // max level
    /// ```
    pub fn increment_severity(&self) -> Self {
        let current = *self as usize;
        Self::from_usize(current + 1).unwrap_or(*self)
    }

    /// Get the next-lowest [`LevelFilter`] from this one.
    ///
    /// If the current [`LevelFilter`] is at the lowest level, the returned [`LevelFilter`] will be the same as the current one.
    ///
    /// # Examples
    ///
    /// ```
    /// use mw_log::LevelFilter;
    ///
    /// let level_filter = LevelFilter::Info;
    ///
    /// assert_eq!(LevelFilter::Warn, level_filter.decrement_severity());
    /// assert_eq!(LevelFilter::Error, level_filter.decrement_severity().decrement_severity());
    /// assert_eq!(LevelFilter::Fatal, level_filter.decrement_severity().decrement_severity().decrement_severity());
    /// assert_eq!(LevelFilter::Off, level_filter.decrement_severity().decrement_severity().decrement_severity().decrement_severity());
    /// assert_eq!(LevelFilter::Off, level_filter.decrement_severity().decrement_severity().decrement_severity().decrement_severity().decrement_severity()); // min level
    /// ```
    pub fn decrement_severity(&self) -> Self {
        let current = *self as usize;
        Self::from_usize(current.saturating_sub(1)).unwrap_or(*self)
    }
}

/// The "payload" of a log message.
#[derive(Clone)]
pub struct Record<'a> {
    metadata: Metadata<'a>,
    args: Arguments<'a>,
    module_path: Option<&'a str>,
    file: Option<&'a str>,
    line: Option<u32>,
}

impl<'a> Record<'a> {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> RecordBuilder<'a> {
        RecordBuilder::new()
    }

    /// The message body.
    #[inline]
    pub fn args(&self) -> &Arguments<'a> {
        &self.args
    }

    /// Metadata about the log directive.
    #[inline]
    pub fn metadata(&self) -> &Metadata<'a> {
        &self.metadata
    }

    /// The verbosity level of the message.
    #[inline]
    pub fn level(&self) -> Level {
        self.metadata.level()
    }

    /// The name of the context of the directive.
    #[inline]
    pub fn context(&self) -> &'a str {
        self.metadata.context()
    }

    /// The module path of the message.
    #[inline]
    pub fn module_path(&self) -> Option<&'a str> {
        self.module_path
    }

    /// The source file containing the message.
    #[inline]
    pub fn file(&self) -> Option<&'a str> {
        self.file
    }

    /// The line containing the message.
    #[inline]
    pub fn line(&self) -> Option<u32> {
        self.line
    }
}

/// Builder for [`Record`].
pub struct RecordBuilder<'a> {
    record: Record<'a>,
}

impl<'a> RecordBuilder<'a> {
    /// Construct new [`RecordBuilder`].
    ///
    /// The default options are:
    ///
    /// - `args`: [`mw_log_format_args!("")`]
    /// - `metadata`: [`Metadata::builder().build()`]
    /// - `module_path`: `None`
    /// - `file`: `None`
    /// - `line`: `None`
    #[inline]
    pub fn new() -> RecordBuilder<'a> {
        // Fix for self-reference in `mw_log_format_args`.
        use crate as mw_log;
        RecordBuilder {
            record: Record {
                args: mw_log_format_args!(""),
                metadata: Metadata::builder().build(),
                module_path: None,
                file: None,
                line: None,
            },
        }
    }

    /// Set [`Record::args`].
    #[inline]
    pub fn args(&mut self, args: Arguments<'a>) -> &mut RecordBuilder<'a> {
        self.record.args = args;
        self
    }

    /// Set [`Record::metadata`].
    #[inline]
    pub fn metadata(&mut self, metadata: Metadata<'a>) -> &mut RecordBuilder<'a> {
        self.record.metadata = metadata;
        self
    }

    /// Set [`Metadata::level`].
    #[inline]
    pub fn level(&mut self, level: Level) -> &mut RecordBuilder<'a> {
        self.record.metadata.level = level;
        self
    }

    /// Set [`Metadata::context`].
    #[inline]
    pub fn context(&mut self, context: &'a str) -> &mut RecordBuilder<'a> {
        self.record.metadata.context = context;
        self
    }

    /// Set [`Record::module_path`].
    #[inline]
    pub fn module_path(&mut self, path: Option<&'a str>) -> &mut RecordBuilder<'a> {
        self.record.module_path = path;
        self
    }

    /// Set [`Record::file`].
    #[inline]
    pub fn file(&mut self, file: Option<&'a str>) -> &mut RecordBuilder<'a> {
        self.record.file = file;
        self
    }

    /// Set [`Record::line`].
    #[inline]
    pub fn line(&mut self, line: Option<u32>) -> &mut RecordBuilder<'a> {
        self.record.line = line;
        self
    }

    /// Invoke the builder and return a [`Record`].
    #[inline]
    pub fn build(&self) -> Record<'a> {
        self.record.clone()
    }
}

impl Default for RecordBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata about a log message.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Metadata<'a> {
    level: Level,
    context: &'a str,
}

impl<'a> Metadata<'a> {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> MetadataBuilder<'a> {
        MetadataBuilder::new()
    }

    /// The verbosity level of the message.
    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    /// The name of the context of the directive.
    #[inline]
    pub fn context(&self) -> &'a str {
        self.context
    }
}

/// Builder for [`Metadata`].
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MetadataBuilder<'a> {
    metadata: Metadata<'a>,
}

impl<'a> MetadataBuilder<'a> {
    /// Construct a new [`MetadataBuilder`].
    ///
    /// The default options are:
    ///
    /// - `level`: `Level::Info`
    /// - `context`: `""`
    #[inline]
    pub fn new() -> MetadataBuilder<'a> {
        MetadataBuilder {
            metadata: Metadata {
                level: Level::Info,
                context: "",
            },
        }
    }

    /// Setter for [`Metadata::level`].
    #[inline]
    pub fn level(&mut self, arg: Level) -> &mut MetadataBuilder<'a> {
        self.metadata.level = arg;
        self
    }

    /// Setter for [`Metadata::context`].
    #[inline]
    pub fn context(&mut self, context: &'a str) -> &mut MetadataBuilder<'a> {
        self.metadata.context = context;
        self
    }

    /// Returns a [`Metadata`] object.
    #[inline]
    pub fn build(&self) -> Metadata<'a> {
        self.metadata.clone()
    }
}

impl Default for MetadataBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// A trait encapsulating the operations required of a logger.
pub trait Log: Sync + Send {
    /// Determines if a log message with the specified metadata would be logged.
    fn enabled(&self, metadata: &Metadata) -> bool;

    /// Logs the [`Record`].
    fn log(&self, record: &Record);

    /// Flushes any buffered records.
    fn flush(&self);
}

/// A dummy initial value for LOGGER.
struct NopLogger;

impl Log for NopLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        false
    }

    fn log(&self, _: &Record) {}

    fn flush(&self) {}
}

impl<T> Log for &'_ T
where
    T: ?Sized + Log,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        (**self).enabled(metadata)
    }

    fn log(&self, record: &Record) {
        (**self).log(record);
    }
    fn flush(&self) {
        (**self).flush();
    }
}

impl<T> Log for std::boxed::Box<T>
where
    T: ?Sized + Log,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.as_ref().enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.as_ref().log(record);
    }
    fn flush(&self) {
        self.as_ref().flush();
    }
}

impl<T> Log for std::sync::Arc<T>
where
    T: ?Sized + Log,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.as_ref().enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.as_ref().log(record);
    }
    fn flush(&self) {
        self.as_ref().flush();
    }
}

/// Sets the global maximum log level.
///
/// Generally, this should only be called by the active logging implementation.
///
/// Note that [`LevelFilter::Trace`] is the maximum level, because it provides the maximum amount of detail in the emitted logs.
#[inline]
pub fn set_max_level(level: LevelFilter) {
    MAX_LOG_LEVEL_FILTER.store(level as usize, Ordering::Relaxed);
}

/// Returns the current maximum log level.
///
/// Logging macros check this value and discard any message logged at a higher level.
/// The maximum log level is set by the [`set_max_level`] function.
#[inline(always)]
pub fn max_level() -> LevelFilter {
    // Since `LevelFilter` is `repr(usize)`,
    // this transmute is sound if and only if `MAX_LOG_LEVEL_FILTER`
    // is set to a usize that is a valid discriminant for `LevelFilter`.
    // Since `MAX_LOG_LEVEL_FILTER` is private, the only time it's set
    // is by `set_max_level` above, i.e. by casting a `LevelFilter` to `usize`.
    // So any usize stored in `MAX_LOG_LEVEL_FILTER` is a valid discriminant.
    unsafe { mem::transmute(MAX_LOG_LEVEL_FILTER.load(Ordering::Relaxed)) }
}

/// Sets the global logger to a `Box<Log>`.
///
/// This is a simple convenience wrapper over [`set_logger`], which takes a `Box<Log>` rather than a `&'static Log`.
/// See the documentation for [`set_logger`] for more details.
///
/// # Errors
///
/// An error is returned if a logger has already been set.
pub fn set_boxed_logger(logger: Box<dyn Log>) -> Result<(), SetLoggerError> {
    set_logger_inner(|| Box::leak(logger))
}

/// Sets the global logger to a `&'static Log`.
///
/// This function may only be called once in the lifetime of a program.
/// Any log events that occur before the call to [`set_logger`] completes will be ignored.
///
/// This function does not typically need to be called manually.
/// Logger implementations should provide an initialization method that installs the logger internally.
///
/// # Errors
///
/// An error is returned if a logger has already been set.
pub fn set_logger(logger: &'static dyn Log) -> Result<(), SetLoggerError> {
    set_logger_inner(|| logger)
}

fn set_logger_inner<F>(make_logger: F) -> Result<(), SetLoggerError>
where
    F: FnOnce() -> &'static dyn Log,
{
    match STATE.compare_exchange(
        UNINITIALIZED,
        INITIALIZING,
        Ordering::Acquire,
        Ordering::Relaxed,
    ) {
        Ok(UNINITIALIZED) => {
            unsafe {
                LOGGER = make_logger();
            }
            STATE.store(INITIALIZED, Ordering::Release);
            Ok(())
        }
        Err(INITIALIZING) => {
            while STATE.load(Ordering::Relaxed) == INITIALIZING {
                core::hint::spin_loop();
            }
            Err(SetLoggerError(()))
        }
        _ => Err(SetLoggerError(())),
    }
}

/// The type returned by [`set_logger`] if [`set_logger`] has already been called.
pub struct SetLoggerError(());

impl core::fmt::Display for SetLoggerError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_str(SET_LOGGER_ERROR)
    }
}

/// The type returned by [`core::str::FromStr::from_str`] implementations when the string doesn't match any of the log levels.
#[derive(PartialEq, Eq)]
pub struct ParseLevelError(());

impl core::fmt::Display for ParseLevelError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_str(LEVEL_PARSE_ERROR)
    }
}

/// Returns a reference to the logger.
///
/// If a logger has not been set, a no-op implementation is returned.
pub fn logger() -> &'static dyn Log {
    // Acquire memory ordering guarantees that current thread would see any
    // memory writes that happened before store of the value
    // into `STATE` with memory ordering `Release` or stronger.
    //
    // Since the value `INITIALIZED` is written only after `LOGGER` was
    // initialized, observing it after `Acquire` load here makes both
    // write to the `LOGGER` static and initialization of the logger
    // internal state synchronized with current thread.
    if STATE.load(Ordering::Acquire) != INITIALIZED {
        static NOP: NopLogger = NopLogger;
        &NOP
    } else {
        unsafe { LOGGER }
    }
}

// WARNING: this is not part of the crate's public API and is subject to change at any time
#[doc(hidden)]
pub mod __private_api;

/// The statically resolved maximum log level.
///
/// See the crate level documentation for information on how to configure this.
///
/// This value is checked by the log macros, but not by the `Logger` returned by the [`logger`] function.
/// Code that manually calls functions on that value should compare the level against this value.
pub const STATIC_MAX_LEVEL: LevelFilter = match cfg!(debug_assertions) {
    false if cfg!(feature = "release_max_level_off") => LevelFilter::Off,
    false if cfg!(feature = "release_max_level_fatal") => LevelFilter::Fatal,
    false if cfg!(feature = "release_max_level_error") => LevelFilter::Error,
    false if cfg!(feature = "release_max_level_warn") => LevelFilter::Warn,
    false if cfg!(feature = "release_max_level_info") => LevelFilter::Info,
    false if cfg!(feature = "release_max_level_debug") => LevelFilter::Debug,
    false if cfg!(feature = "release_max_level_trace") => LevelFilter::Trace,
    _ if cfg!(feature = "max_level_off") => LevelFilter::Off,
    _ if cfg!(feature = "max_level_fatal") => LevelFilter::Fatal,
    _ if cfg!(feature = "max_level_error") => LevelFilter::Error,
    _ if cfg!(feature = "max_level_warn") => LevelFilter::Warn,
    _ if cfg!(feature = "max_level_info") => LevelFilter::Info,
    _ if cfg!(feature = "max_level_debug") => LevelFilter::Debug,
    _ => LevelFilter::Trace,
};

#[cfg(test)]
mod level_tests {
    use super::{Level, LevelFilter, ParseLevelError};
    use core::iter::zip;
    use std::collections::HashSet;

    #[test]
    fn test_level_partial_eq_with_level_filter() {
        // Pairs that should result in true.
        let matching_pairs = HashSet::from([
            (Level::Fatal, LevelFilter::Fatal),
            (Level::Error, LevelFilter::Error),
            (Level::Warn, LevelFilter::Warn),
            (Level::Info, LevelFilter::Info),
            (Level::Debug, LevelFilter::Debug),
            (Level::Trace, LevelFilter::Trace),
        ]);

        // Iterate through levels.
        for level in Level::iter() {
            // Iterate through level filters.
            for level_filter in LevelFilter::iter() {
                let is_matching = matching_pairs.contains(&(level, level_filter));
                assert_eq!(level.eq(&level_filter), is_matching);
            }
        }
    }

    #[test]
    fn test_level_partial_cmp_with_level_filter() {
        // Iterate through levels.
        for level in Level::iter() {
            let matching_filter = level.to_level_filter();
            // Iterate through level filters.
            for level_filter in LevelFilter::iter() {
                if matching_filter < level_filter {
                    assert!(level < level_filter);
                } else if matching_filter > level_filter {
                    assert!(level > level_filter);
                } else {
                    assert!(level == level_filter);
                }
            }
        }
    }

    #[test]
    fn test_level_from_str() {
        let tests = [
            ("OFF", Err(ParseLevelError(()))),
            ("fatal", Ok(Level::Fatal)),
            ("error", Ok(Level::Error)),
            ("warn", Ok(Level::Warn)),
            ("info", Ok(Level::Info)),
            ("debug", Ok(Level::Debug)),
            ("trace", Ok(Level::Trace)),
            ("FATAL", Ok(Level::Fatal)),
            ("ERROR", Ok(Level::Error)),
            ("WARN", Ok(Level::Warn)),
            ("INFO", Ok(Level::Info)),
            ("DEBUG", Ok(Level::Debug)),
            ("TRACE", Ok(Level::Trace)),
            ("asdf", Err(ParseLevelError(()))),
        ];
        for &(s, ref expected) in &tests {
            assert!(expected == &s.parse());
        }
    }

    #[test]
    fn test_level_display() {
        assert_eq!("FATAL", Level::Fatal.to_string());
        assert_eq!("ERROR", Level::Error.to_string());
        assert_eq!("WARN", Level::Warn.to_string());
        assert_eq!("INFO", Level::Info.to_string());
        assert_eq!("DEBUG", Level::Debug.to_string());
        assert_eq!("TRACE", Level::Trace.to_string());
    }

    #[test]
    fn test_level_from_usize() {
        let tests = [
            None,
            Some(Level::Fatal),
            Some(Level::Error),
            Some(Level::Warn),
            Some(Level::Info),
            Some(Level::Debug),
            Some(Level::Trace),
            None,
        ];
        for (i, expected) in tests.iter().enumerate() {
            assert_eq!(expected, &Level::from_usize(i));
        }
    }

    #[test]
    fn test_level_max() {
        assert_eq!(Level::max(), Level::Trace);
    }

    #[test]
    fn test_level_to_level_filter() {
        let tests = [
            (Level::Fatal, LevelFilter::Fatal),
            (Level::Error, LevelFilter::Error),
            (Level::Warn, LevelFilter::Warn),
            (Level::Info, LevelFilter::Info),
            (Level::Debug, LevelFilter::Debug),
            (Level::Trace, LevelFilter::Trace),
        ];
        for (level, level_filter) in tests {
            assert_eq!(level_filter, level.to_level_filter());
        }
    }

    #[test]
    fn test_level_as_str() {
        let tests = &[
            (Level::Fatal, "FATAL"),
            (Level::Error, "ERROR"),
            (Level::Warn, "WARN"),
            (Level::Info, "INFO"),
            (Level::Debug, "DEBUG"),
            (Level::Trace, "TRACE"),
        ];
        for (input, expected) in tests {
            assert_eq!(*expected, input.as_str());
        }
    }

    #[test]
    fn test_level_iter() {
        let expected_order = [
            Level::Fatal,
            Level::Error,
            Level::Warn,
            Level::Info,
            Level::Debug,
            Level::Trace,
        ];

        let act_it = Level::iter();
        let exp_it = expected_order.iter();
        for (exp, act) in zip(exp_it, act_it) {
            assert_eq!(*exp, act);
        }
    }

    #[test]
    fn test_level_increment_severity() {
        let info = Level::Info;
        let up = info.increment_severity();
        assert_eq!(up, Level::Debug);

        let trace = Level::Trace;
        let up = trace.increment_severity();
        // trace is already highest level
        assert_eq!(up, trace);
    }
}

#[cfg(test)]
mod level_filter_tests {
    use super::{Level, LevelFilter, ParseLevelError};
    use core::iter::zip;
    use std::collections::HashSet;

    #[test]
    fn test_level_decrement_severity() {
        let info = Level::Info;
        let down = info.decrement_severity();
        assert_eq!(down, Level::Warn);

        let fatal = Level::Fatal;
        let down = fatal.decrement_severity();
        // fatal is already lowest level
        assert_eq!(down, fatal);
    }

    #[test]
    fn test_level_filter_partial_eq_with_level() {
        // Pairs that should result in true.
        let matching_pairs = HashSet::from([
            (LevelFilter::Fatal, Level::Fatal),
            (LevelFilter::Error, Level::Error),
            (LevelFilter::Warn, Level::Warn),
            (LevelFilter::Info, Level::Info),
            (LevelFilter::Debug, Level::Debug),
            (LevelFilter::Trace, Level::Trace),
        ]);

        // Iterate through level filters.
        for level_filter in LevelFilter::iter() {
            // Iterate through levels.
            for level in Level::iter() {
                let is_matching = matching_pairs.contains(&(level_filter, level));
                assert_eq!(level_filter.eq(&level), is_matching)
            }
        }
    }

    #[test]
    fn test_level_filter_partial_cmp_with_level() {
        // Iterate through level filters.
        // Skip `LevelFilter::Off`.
        for level_filter in LevelFilter::iter().skip(1) {
            let matching_level = level_filter.to_level().unwrap();
            // Iterate through levels.
            for level in Level::iter() {
                if matching_level < level {
                    assert!(level_filter < level);
                } else if matching_level > level {
                    assert!(level_filter > level);
                } else {
                    assert!(level_filter == level);
                }
            }
        }
    }

    #[test]
    fn test_level_filter_from_str() {
        let tests = [
            ("off", Ok(LevelFilter::Off)),
            ("fatal", Ok(LevelFilter::Fatal)),
            ("error", Ok(LevelFilter::Error)),
            ("warn", Ok(LevelFilter::Warn)),
            ("info", Ok(LevelFilter::Info)),
            ("debug", Ok(LevelFilter::Debug)),
            ("trace", Ok(LevelFilter::Trace)),
            ("OFF", Ok(LevelFilter::Off)),
            ("FATAL", Ok(LevelFilter::Fatal)),
            ("ERROR", Ok(LevelFilter::Error)),
            ("WARN", Ok(LevelFilter::Warn)),
            ("INFO", Ok(LevelFilter::Info)),
            ("DEBUG", Ok(LevelFilter::Debug)),
            ("TRACE", Ok(LevelFilter::Trace)),
            ("asdf", Err(ParseLevelError(()))),
        ];
        for &(s, ref expected) in &tests {
            assert!(expected == &s.parse());
        }
    }

    #[test]
    fn test_level_filter_display() {
        assert_eq!("OFF", LevelFilter::Off.to_string());
        assert_eq!("FATAL", LevelFilter::Fatal.to_string());
        assert_eq!("ERROR", LevelFilter::Error.to_string());
        assert_eq!("WARN", LevelFilter::Warn.to_string());
        assert_eq!("INFO", LevelFilter::Info.to_string());
        assert_eq!("DEBUG", LevelFilter::Debug.to_string());
        assert_eq!("TRACE", LevelFilter::Trace.to_string());
        assert_eq!("ERROR", LevelFilter::Error.to_string());
    }

    #[test]
    fn test_level_filter_from_usize() {
        let tests = [
            Some(LevelFilter::Off),
            Some(LevelFilter::Fatal),
            Some(LevelFilter::Error),
            Some(LevelFilter::Warn),
            Some(LevelFilter::Info),
            Some(LevelFilter::Debug),
            Some(LevelFilter::Trace),
            None,
        ];
        for (i, expected) in tests.iter().enumerate() {
            assert_eq!(expected, &LevelFilter::from_usize(i));
        }
    }

    #[test]
    fn test_level_filter_max() {
        assert_eq!(LevelFilter::max(), LevelFilter::Trace);
    }

    #[test]
    fn test_level_filter_to_level() {
        let tests = [
            (LevelFilter::Off, None),
            (LevelFilter::Fatal, Some(Level::Fatal)),
            (LevelFilter::Error, Some(Level::Error)),
            (LevelFilter::Warn, Some(Level::Warn)),
            (LevelFilter::Info, Some(Level::Info)),
            (LevelFilter::Debug, Some(Level::Debug)),
            (LevelFilter::Trace, Some(Level::Trace)),
        ];
        for (level_filter, level) in tests {
            assert_eq!(level, level_filter.to_level());
        }
    }

    #[test]
    fn test_level_filter_as_str() {
        let tests = &[
            (LevelFilter::Off, "OFF"),
            (LevelFilter::Fatal, "FATAL"),
            (LevelFilter::Error, "ERROR"),
            (LevelFilter::Warn, "WARN"),
            (LevelFilter::Info, "INFO"),
            (LevelFilter::Debug, "DEBUG"),
            (LevelFilter::Trace, "TRACE"),
        ];
        for (input, expected) in tests {
            assert_eq!(*expected, input.as_str());
        }
    }

    #[test]
    fn test_level_filter_iter() {
        let expected_order = [
            LevelFilter::Off,
            LevelFilter::Fatal,
            LevelFilter::Error,
            LevelFilter::Warn,
            LevelFilter::Info,
            LevelFilter::Debug,
            LevelFilter::Trace,
        ];

        let act_it = LevelFilter::iter();
        let exp_it = expected_order.iter();
        for (exp, act) in zip(exp_it, act_it) {
            assert_eq!(*exp, act);
        }
    }

    #[test]
    fn test_level_filter_increment_severity() {
        let info = LevelFilter::Info;
        let up = info.increment_severity();
        assert_eq!(up, LevelFilter::Debug);

        let trace = LevelFilter::Trace;
        let up = trace.increment_severity();
        // trace is already highest level
        assert_eq!(up, trace);
    }

    #[test]
    fn test_level_filter_decrement_severity() {
        let info = LevelFilter::Info;
        let down = info.decrement_severity();
        assert_eq!(down, LevelFilter::Warn);

        let fatal = LevelFilter::Fatal;
        let down = fatal.decrement_severity();
        assert_eq!(down, LevelFilter::Off);
        // Off is already the lowest
        assert_eq!(down.decrement_severity(), down);
    }
}

#[cfg(test)]
mod record_tests {
    use super::{Level, MetadataBuilder, Record, RecordBuilder};
    // Fix for self-reference in `mw_log_format_args`.
    use crate as mw_log;
    use mw_log_fmt_macro::mw_log_format_args;

    #[test]
    fn test_record_builder_convenience_method() {
        let metadata = MetadataBuilder::new().build();
        let record = Record::builder().build();
        assert_eq!(record.args().0.len(), 0);
        assert!(record.metadata() == &metadata);
        assert_eq!(record.level(), metadata.level());
        assert_eq!(record.context(), metadata.context());
        assert_eq!(record.module_path(), None);
        assert_eq!(record.file(), None);
        assert_eq!(record.line(), None);
    }

    #[test]
    fn test_record_builder_new_no_params() {
        let metadata = MetadataBuilder::new().build();
        let record = RecordBuilder::new().build();
        assert_eq!(record.args().0.len(), 0);
        assert!(record.metadata() == &metadata);
        assert_eq!(record.level(), metadata.level());
        assert_eq!(record.context(), metadata.context());
        assert_eq!(record.module_path(), None);
        assert_eq!(record.file(), None);
        assert_eq!(record.line(), None);
    }

    #[test]
    fn test_record_builder_default() {
        let metadata = MetadataBuilder::new().build();
        let record = RecordBuilder::default().build();
        assert_eq!(record.args().0.len(), 0);
        assert!(record.metadata() == &metadata);
        assert_eq!(record.level(), metadata.level());
        assert_eq!(record.context(), metadata.context());
        assert_eq!(record.module_path(), None);
        assert_eq!(record.file(), None);
        assert_eq!(record.line(), None);
    }

    #[test]
    fn test_record_args() {
        let metadata = MetadataBuilder::new().build();
        let args = mw_log_format_args!("test_string_{}", 123);
        let record = RecordBuilder::new().args(args).build();
        assert_eq!(record.args().0.len(), 2);
        assert!(record.metadata() == &metadata);
        assert_eq!(record.level(), metadata.level());
        assert_eq!(record.context(), metadata.context());
        assert_eq!(record.module_path(), None);
        assert_eq!(record.file(), None);
        assert_eq!(record.line(), None);
    }

    #[test]
    fn test_record_metadata() {
        let metadata = MetadataBuilder::new()
            .level(Level::Debug)
            .context("context")
            .build();
        let record = RecordBuilder::new().metadata(metadata.clone()).build();
        assert_eq!(record.args().0.len(), 0);
        assert!(record.metadata() == &metadata);
        assert_eq!(record.level(), metadata.level());
        assert_eq!(record.context(), metadata.context());
        assert_eq!(record.module_path(), None);
        assert_eq!(record.file(), None);
        assert_eq!(record.line(), None);
    }

    #[test]
    fn test_record_level() {
        let level = Level::Error;
        let metadata = MetadataBuilder::new().level(level).build();
        let record = RecordBuilder::new().level(level).build();
        assert_eq!(record.args().0.len(), 0);
        assert!(record.metadata() == &metadata);
        assert_eq!(record.level(), level);
        assert_eq!(record.context(), metadata.context());
        assert_eq!(record.module_path(), None);
        assert_eq!(record.file(), None);
        assert_eq!(record.line(), None);
    }

    #[test]
    fn test_record_context() {
        let context = "context";
        let metadata = MetadataBuilder::new().context(context).build();
        let record = RecordBuilder::new().context(context).build();
        assert_eq!(record.args().0.len(), 0);
        assert!(record.metadata() == &metadata);
        assert_eq!(record.level(), metadata.level());
        assert_eq!(record.context(), context);
        assert_eq!(record.module_path(), None);
        assert_eq!(record.file(), None);
        assert_eq!(record.line(), None);
    }

    #[test]
    fn test_record_module_path() {
        let metadata = MetadataBuilder::new().build();
        let module_path = "module_path";
        let record = RecordBuilder::new().module_path(Some(module_path)).build();
        assert_eq!(record.args().0.len(), 0);
        assert!(record.metadata() == &metadata);
        assert_eq!(record.level(), metadata.level());
        assert_eq!(record.context(), metadata.context());
        assert_eq!(record.module_path(), Some(module_path));
        assert_eq!(record.file(), None);
        assert_eq!(record.line(), None);
    }

    #[test]
    fn test_record_file() {
        let metadata = MetadataBuilder::new().build();
        let file = "file";
        let record = RecordBuilder::new().file(Some(file)).build();
        assert_eq!(record.args().0.len(), 0);
        assert!(record.metadata() == &metadata);
        assert_eq!(record.level(), metadata.level());
        assert_eq!(record.context(), metadata.context());
        assert_eq!(record.module_path(), None);
        assert_eq!(record.file(), Some(file));
        assert_eq!(record.line(), None);
    }

    #[test]
    fn test_record_line() {
        let metadata = MetadataBuilder::new().build();
        let line_num = 321u32;
        let record = RecordBuilder::new().line(Some(line_num)).build();
        assert_eq!(record.args().0.len(), 0);
        assert!(record.metadata() == &metadata);
        assert_eq!(record.level(), metadata.level());
        assert_eq!(record.context(), metadata.context());
        assert_eq!(record.module_path(), None);
        assert_eq!(record.file(), None);
        assert_eq!(record.line(), Some(line_num));
    }

    #[test]
    fn test_record_chained_metadata() {
        let context = "context";
        let level = Level::Trace;
        let metadata = MetadataBuilder::new().context(context).level(level).build();

        let args = mw_log_format_args!("test_string_{}", 123);
        let module_path = Some("module_path");
        let file = Some("file");
        let line_num = Some(123u32);
        let record = RecordBuilder::new()
            .args(args)
            .metadata(metadata.clone())
            .module_path(module_path)
            .file(file)
            .line(line_num)
            .build();
        assert_eq!(record.args().0.len(), 2);
        assert!(record.metadata() == &metadata);
        assert_eq!(record.level(), metadata.level());
        assert_eq!(record.context(), metadata.context());
        assert_eq!(record.module_path(), module_path);
        assert_eq!(record.file(), file);
        assert_eq!(record.line(), line_num);
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::{Level, Metadata, MetadataBuilder};

    #[test]
    fn test_metadata_builder_convenience_method() {
        let metadata = Metadata::builder().build();
        assert_eq!(metadata.level(), Level::Info);
        assert_eq!(metadata.context(), "");
    }

    #[test]
    fn test_metadata_builder_new_no_params() {
        let metadata = MetadataBuilder::new().build();
        assert_eq!(metadata.level(), Level::Info);
        assert_eq!(metadata.context(), "");
    }

    #[test]
    fn test_metadata_builder_default() {
        let metadata_new = MetadataBuilder::new().build();
        let metadata_default = MetadataBuilder::default().build();
        assert!(metadata_new == metadata_default);
    }

    #[test]
    fn test_metadata_level() {
        let metadata = MetadataBuilder::new().level(Level::Fatal).build();
        assert_eq!(metadata.level(), Level::Fatal);
        assert_eq!(metadata.context(), "");
    }

    #[test]
    fn test_metadata_context() {
        let context = "test_context";
        let metadata = MetadataBuilder::new().context(context).build();
        assert_eq!(metadata.level(), Level::Info);
        assert_eq!(metadata.context(), context);
    }

    #[test]
    fn test_metadata_chained() {
        let context = "test_context";
        let metadata = MetadataBuilder::new()
            .level(Level::Error)
            .context(context)
            .build();
        assert_eq!(metadata.level(), Level::Error);
        assert_eq!(metadata.context(), context);
    }
}

#[cfg(test)]
mod set_logger_and_logger_tests {
    use crate::NopLogger;

    use super::{set_boxed_logger, set_logger, Log, LOGGER, STATE, UNINITIALIZED};
    use core::sync::atomic::Ordering;
    use std::sync::{LazyLock, Mutex, MutexGuard};

    struct StubLogger;

    impl Log for StubLogger {
        fn enabled(&self, _: &crate::Metadata) -> bool {
            unimplemented!()
        }

        fn log(&self, _: &crate::Record) {
            unimplemented!()
        }

        fn flush(&self) {
            unimplemented!()
        }
    }

    /// Serial test execution mutex.
    static SERIAL_TEST: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    /// Execute test serially with global logger uninitialized.
    fn lock_and_reset<'a>() -> MutexGuard<'a, ()> {
        // Tests in this group must be executed serially.
        let serial_lock: MutexGuard<'a, ()> = SERIAL_TEST.lock().unwrap();

        // Reset logger state.
        unsafe {
            LOGGER = &NopLogger;
        }
        STATE.store(UNINITIALIZED, Ordering::Release);

        serial_lock
    }

    #[test]
    fn test_set_logger_and_logger() {
        let _lock = lock_and_reset();
        assert!(set_logger(&StubLogger).is_ok());
    }

    #[test]
    fn test_set_logger_twice() {
        let _lock = lock_and_reset();
        assert!(set_logger(&StubLogger).is_ok());
        assert!(set_logger(&StubLogger).is_err());
    }

    #[test]
    fn test_set_boxed_logger_and_logger() {
        let _lock = lock_and_reset();
        assert!(set_boxed_logger(Box::new(StubLogger)).is_ok());
    }

    #[test]
    fn test_set_boxed_logger_twice() {
        let _lock = lock_and_reset();
        assert!(set_boxed_logger(Box::new(StubLogger)).is_ok());
        assert!(set_boxed_logger(Box::new(StubLogger)).is_err());
    }
}

#[cfg(test)]
mod other_tests {
    use super::{
        max_level, set_max_level, LevelFilter, Log, ParseLevelError, SetLoggerError,
        STATIC_MAX_LEVEL,
    };

    // Test that the `impl Log for Foo` blocks work
    // This test mostly operates on a type level, so failures will be compile errors
    #[test]
    fn test_foreign_impl() {
        use std::sync::Arc;

        fn assert_is_log<T: Log + ?Sized>() {}

        assert_is_log::<&dyn Log>();

        assert_is_log::<Box<dyn Log>>();

        assert_is_log::<Arc<dyn Log>>();

        // Assert these statements for all T: Log + ?Sized
        #[allow(unused)]
        fn forall<T: Log + ?Sized>() {
            assert_is_log::<Box<T>>();

            assert_is_log::<&T>();

            assert_is_log::<Arc<T>>();
        }
    }

    #[test]
    fn test_max_level_and_set_max_level() {
        // NOTE: `max_level` and `set_max_level` operate on a global state.
        // Changing it affects all tests.

        // Check default value.
        assert_eq!(max_level(), LevelFilter::Off);

        // Set new value and check.
        set_max_level(LevelFilter::Trace);
        assert_eq!(max_level(), LevelFilter::Trace);

        // Reset to original state.
        set_max_level(LevelFilter::Off);
    }

    #[test]
    fn test_set_logger_error_message() {
        let e = SetLoggerError(());
        assert_eq!(
            &e.to_string(),
            "attempted to set a logger after the logging system \
             was already initialized"
        );
    }

    #[test]
    fn test_parse_level_error_message() {
        let e = ParseLevelError(());
        assert_eq!(
            &e.to_string(),
            "attempted to convert a string that doesn't match an existing log level"
        );
    }

    #[test]
    #[cfg_attr(not(debug_assertions), ignore)]
    fn test_static_max_level_debug() {
        if cfg!(feature = "max_level_off") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Off);
        } else if cfg!(feature = "max_level_fatal") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Fatal);
        } else if cfg!(feature = "max_level_error") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Error);
        } else if cfg!(feature = "max_level_warn") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Warn);
        } else if cfg!(feature = "max_level_info") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Info);
        } else if cfg!(feature = "max_level_debug") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Debug);
        } else {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Trace);
        }
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn test_static_max_level_release() {
        if cfg!(feature = "release_max_level_off") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Off);
        } else if cfg!(feature = "release_max_level_fatal") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Fatal);
        } else if cfg!(feature = "release_max_level_error") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Error);
        } else if cfg!(feature = "release_max_level_warn") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Warn);
        } else if cfg!(feature = "release_max_level_info") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Info);
        } else if cfg!(feature = "release_max_level_debug") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Debug);
        } else if cfg!(feature = "release_max_level_trace") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Trace);
        } else if cfg!(feature = "max_level_off") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Off);
        } else if cfg!(feature = "max_level_fatal") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Fatal);
        } else if cfg!(feature = "max_level_error") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Error);
        } else if cfg!(feature = "max_level_warn") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Warn);
        } else if cfg!(feature = "max_level_info") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Info);
        } else if cfg!(feature = "max_level_debug") {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Debug);
        } else {
            assert_eq!(STATIC_MAX_LEVEL, LevelFilter::Trace);
        }
    }
}
