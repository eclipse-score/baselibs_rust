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

//! Implementation of `ScoreDisplay` and `ScoreDebug` for `std` types.

use crate::fmt;
use crate::fmt::*;
use crate::fmt_spec::FormatSpec;
use std::path::{Display, Path, PathBuf};

impl ScoreDisplay for String {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> fmt::Result {
        f.write_str(self, spec)
    }
}

impl ScoreDebug for String {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> fmt::Result {
        f.write_str("\"", spec)?;
        f.write_str(self, spec)?;
        f.write_str("\"", spec)
    }
}

impl ScoreDisplay for &String {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> fmt::Result {
        f.write_str(self, spec)
    }
}

impl ScoreDebug for &String {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> fmt::Result {
        f.write_str("\"", spec)?;
        f.write_str(self, spec)?;
        f.write_str("\"", spec)
    }
}

impl<'a> ScoreDisplay for Display<'a> {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        f.write_str(self.to_string().as_str(), spec)
    }
}

impl<'a> ScoreDebug for Display<'a> {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        f.write_str("\"", spec)?;
        f.write_str(self.to_string().as_str(), spec)?;
        f.write_str("\"", spec)
    }
}

impl ScoreDebug for Path {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        ScoreDebug::fmt(&self.display(), f, spec)
    }
}

impl ScoreDebug for &Path {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        ScoreDebug::fmt(&self.display(), f, spec)
    }
}

impl ScoreDebug for PathBuf {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        ScoreDebug::fmt(&self.display(), f, spec)
    }
}

impl ScoreDebug for &PathBuf {
    fn fmt(&self, f: Writer, spec: &FormatSpec) -> Result {
        ScoreDebug::fmt(&self.display(), f, spec)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{common_test_debug, common_test_display};
    use std::path::{Path, PathBuf};

    #[test]
    fn test_string_display() {
        common_test_display(String::from("test"));
    }

    #[test]
    fn test_string_debug() {
        common_test_debug(String::from("test"));
    }

    #[test]
    fn test_string_ref_display() {
        let value = String::from("test");
        common_test_display(&value);
    }

    #[test]
    fn test_string_ref_debug() {
        let value = String::from("test");
        common_test_debug(&value);
    }

    #[test]
    fn test_path_display_display() {
        common_test_display(Path::new("/tmp/test_path").display());
    }

    #[test]
    fn test_path_display_debug() {
        common_test_debug(Path::new("/tmp/test_path").display());
    }

    #[test]
    fn test_path_ref_debug() {
        common_test_debug(Path::new("/tmp/test_path"));
    }

    #[test]
    fn test_pathbuf_debug() {
        common_test_debug(PathBuf::from("/tmp/test_path"));
    }

    #[test]
    fn test_pathbuf_ref_debug() {
        let value = PathBuf::from("/tmp/test_path");
        common_test_debug(&value);
    }
}
