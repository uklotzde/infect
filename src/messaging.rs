// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use futures::channel::mpsc;

use crate::Message;

pub type MessageSender<Intent, Effect> = mpsc::Sender<Message<Intent, Effect>>;
pub type MessageReceiver<Intent, Effect> = mpsc::Receiver<Message<Intent, Effect>>;
pub type MessageChannel<Intent, Effect> = (
    MessageSender<Intent, Effect>,
    MessageReceiver<Intent, Effect>,
);

/// Create a buffered message channel with limited capacity.
#[must_use]
pub fn message_channel<Intent, Effect>(
    capacity: usize,
) -> (
    MessageSender<Intent, Effect>,
    MessageReceiver<Intent, Effect>,
) {
    mpsc::channel(capacity)
}

pub fn send_message<Intent: fmt::Debug, Effect: fmt::Debug>(
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
