// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use futures::channel::mpsc;

use crate::Message;

/// Message sender for submitting messages
pub type MessageSender<Intent, Effect> = mpsc::Sender<Message<Intent, Effect>>;

/// Message receiver for consuming messages
pub type MessageReceiver<Intent, Effect> = mpsc::Receiver<Message<Intent, Effect>>;

/// Buffered, MPSC message channel
pub type MessageChannel<Intent, Effect> = (
    MessageSender<Intent, Effect>,
    MessageReceiver<Intent, Effect>,
);

/// Create a buffered, MPSC message channel with limited capacity
///
/// FIFO queue of sent messages that are consumed by a single [`MessageReceiver`].
#[must_use]
pub fn message_channel<Intent, Effect>(
    capacity: usize,
) -> (
    MessageSender<Intent, Effect>,
    MessageReceiver<Intent, Effect>,
) {
    mpsc::channel(capacity)
}

/// Enqueue a message into the channel
///
/// A utility function that detects and logs unexpected send failures
/// that the submitter should not be bothered with.
///
/// Submitting a message is a fire-and-forget operation that must
/// always succeed. The framework is responsible for dealing with
/// unexpected failures.
pub fn submit_message<Intent: fmt::Debug, Effect: fmt::Debug>(
    message_tx: &mut MessageSender<Intent, Effect>,
    message: impl Into<Message<Intent, Effect>>,
) {
    let message = message.into();
    log::debug!("Sending message: {message:?}");
    if let Err(err) = message_tx.try_send(message) {
        if err.is_disconnected() {
            // No receiver
            log::debug!(
                "Dropping message - channel is closed: {msg:?}",
                msg = err.into_inner()
            );
        } else if err.is_full() {
            log::warn!(
                "Dropping message - channel is full: {msg:?}",
                msg = err.into_inner()
            );
        } else {
            // This code should be unreachable
            log::error!("Failed to send message: {err}");
        }
    }
}
