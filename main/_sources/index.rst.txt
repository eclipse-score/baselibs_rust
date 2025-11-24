..
   # *******************************************************************************
   # Copyright (c) 2025 Contributors to the Eclipse Foundation
   #
   # See the NOTICE file(s) distributed with this work for additional
   # information regarding copyright ownership.
   #
   # This program and the accompanying materials are made available under the
   # terms of the Apache License Version 2.0 which is available at
   # https://www.apache.org/licenses/LICENSE-2.0
   #
   # SPDX-License-Identifier: Apache-2.0
   # *******************************************************************************

Rust Base Libraries
===================

.. toctree::
   :maxdepth: 1
   :glob:
   :titlesonly:

   module_docs/index
   module/*/index

Overview
--------

Foundational Rust libraries providing common functionality for S-CORE modules.

Project Layout
--------------

Project is structured in following manner:

- `README.md`: Short description and build instructions
- `src/`: Source files
- `tests/`: Unit tests (UT) and integration tests (IT)
- `examples/`: Usage examples
- `docs/`: Documentation using `docs-as-code`
- `.github/workflows/`: CI/CD pipelines
- `.vscode`: Recommended VS Code settings
- `.bazelrc`, `.bazelversion`, `MODULE.bazel`, `BUILD`: Bazel configuration and settings
- `Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`: Cargo configuration and settings
- `project_config.bzl`: Project-specific metadata for Bazel macros
- `LICENSE`, `LICENSE.md`: Licensing information
- `CONTRIBUTION.md`: Contribution guidelines
- `NOTICE`: Notices for Eclipse Safe Open Vehicle Core

Quick Start
-----------

To build the module for host platform:

.. code-block:: bash

   bazel build //src/...

To run tests:

.. code-block:: bash

   bazel test //tests/...
