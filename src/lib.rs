// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

//! infect - Intent/Action/Effect

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(rustdoc::broken_intra_doc_links)]
// Repetitions of module/type names occur frequently when using many
// modules for keeping the size of the source files handy. Often
// types have the same name as their parent module.
#![allow(clippy::module_name_repetitions)]
// Repeating the type name in `..Default::default()` expressions
// is not needed since the context is obvious.
#![allow(clippy::default_trait_access)]
// TODO
#![allow(missing_docs)]

pub mod action;
pub use self::action::Action;

pub mod message;
pub use self::message::{
    message_channel, message_loop, Message, MessageChannel, MessageReceiver, MessageSender,
};

pub mod state;
pub use self::state::{state_updated, RenderStateFn, State, StateChanged, StateUpdated};

pub mod task;
pub use self::task::TaskDispatcher;
