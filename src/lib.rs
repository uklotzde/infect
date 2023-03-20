// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

#![allow(rustdoc::invalid_rust_codeblocks)]
#![doc = include_str!("../README.md")]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(clippy::clone_on_ref_ptr)]
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

mod effect;
pub use self::effect::EffectApplied;

mod intent;
pub use self::intent::{IntentAccepted, IntentHandled};

mod message;
pub use self::message::Message;

mod messaging;
pub use self::messaging::{
    message_channel, submit_effect, submit_intent, submit_message, MessageChannel, MessageReceiver,
    MessageSender,
};

mod model;
pub use self::model::{Model, ModelChanged, ModelRender};

mod processing;
pub use self::processing::{consume_messages, process_message, MessageProcessed, MessagesConsumed};

mod task;
pub use self::task::{TaskContext, TaskExecutor};
