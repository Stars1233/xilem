// Copyright 2021 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

// TODO - See https://github.com/linebender/xilem/issues/336

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::dbg_macro,
    reason = "Deferred: Tests need to be refactored"
)]

mod ime_focused;
mod layout;
mod lifecycle_basic;
mod lifecycle_disable;
mod lifecycle_focus;
mod safety_rails;
mod status_change;
mod transforms;
mod widget_tree;
