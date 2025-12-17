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

.. _component_detailed_design_log:

Detailed Design
###############

.. document:: Log Detailed Design
   :id: doc__log_detailed_design
   :status: draft
   :safety: ASIL_B
   :security: NO
   :realizes: wp__sw_implementation
   :tags: log


Detailed Design for Component: Log
==================================

Description
-----------

Log component consists of three units:

- `mw_log` - modelled after `log` Rust library.
- `mw_log_fmt` - replacement for `core::fmt` provided by Rust core library.
- `mw_log_macro` - replacement for `format_args` macro provided by Rust compiler.

Most common approach in Rust is that formatting always results in a string.
This means that the `log` library always receives a pre-formatted string.

Such approach is incompatible with the expectation that log sink is not always text-based.
Log component design is no longer string-based, and data frames can consist of multiple types.
Value is passed along with formatting options to the backend.

Rationale Behind Decomposition into Units
******************************************

All units provide an interface or an implementation to a well defined functionality.
Those units are not described in architecture, as they all form a monolithic logging frontend.

Such frontend shall remain transparent replacement to common functionalities provided by Rust.

Static Diagrams for Unit Interactions
-------------------------------------

.. dd_sta:: Log class diagram
   :id: dd_sta__log__class_diagram
   :security: NO
   :safety: ASIL_B
   :status: valid
   :implements: comp_req__log__placeholder
   :satisfies: comp_arc_sta__log__static_view

   .. uml:: _assets/class_diagram.puml

Dynamic Diagrams for Unit Interactions
--------------------------------------

.. dd_dyn:: Log operation
   :id: dd_dyn__log__log_op
   :security: NO
   :safety: ASIL_B
   :status: valid
   :implements: comp_req__log__placeholder
   :satisfies: comp_arc_sta__log__static_view

   .. uml:: _assets/log_op.puml

.. dd_dyn:: Log to level
   :id: dd_dyn__log__log_to_level
   :security: NO
   :safety: ASIL_B
   :status: valid
   :implements: comp_req__log__placeholder
   :satisfies: comp_arc_sta__log__static_view

   .. uml:: _assets/log_to_level.puml
