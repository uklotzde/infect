// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use futures_channel::mpsc;

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
/// FIFO queue of sent messages that are consumed by a single
/// [`MessageReceiver`].
#[must_use]
pub fn message_channel<Intent, Effect>(
    capacity: usize,
) -> (
    MessageSender<Intent, Effect>,
    MessageReceiver<Intent, Effect>,
) {
    mpsc::channel(capacity)
}

/// Domain-specific wrapper around a [`MessageSender`]
#[derive(Debug)]
pub struct MessagePort<Intent, Effect> {
    message_tx: MessageSender<Intent, Effect>,
}

impl<Intent, Effect> MessagePort<Intent, Effect> {
    /// Create a new instance
    #[must_use]
    pub fn new(message_tx: MessageSender<Intent, Effect>) -> Self {
        Self { message_tx }
    }

    /// Obtain the inner [`MessageSender`] for the channel
    #[must_use]
    pub fn into_inner(self) -> MessageSender<Intent, Effect> {
        let Self { message_tx } = self;
        message_tx
    }
}

impl<Intent, Effect> MessagePort<Intent, Effect>
where
    Intent: fmt::Debug,
    Effect: fmt::Debug,
{
    /// Enqueue a message into the channel
    ///
    /// A utility function that detects and logs unexpected send failures
    /// that the submitter should not be bothered with.
    ///
    /// Submitting a message is a fire-and-forget operation that must
    /// always succeed. The framework is responsible for dealing with
    /// unexpected failures.
    pub fn submit_message(&mut self, message: impl Into<Message<Intent, Effect>>) {
        let message = message.into();
        log::debug!("Sending message: {message:?}");
        if let Err(err) = self.message_tx.try_send(message) {
            if err.is_disconnected() {
                // No receiver
                log::debug!(
                    "Dropping message - channel is closed: {message:?}",
                    message = err.into_inner()
                );
            } else if err.is_full() {
                log::warn!(
                    "Dropping message - channel is full: {message:?}",
                    message = err.into_inner()
                );
            } else {
                // This code should be unreachable
                log::error!("Failed to send message: {err}");
            }
        }
    }

    /// Submit an intent
    ///
    /// See also: [`Self::submit_message`]
    pub fn submit_intent(&mut self, intent: impl Into<Intent>) {
        self.submit_message(Message::Intent(intent.into()));
    }

    /// Submit an effect
    ///
    /// See also: [`Self::submit_message`]
    pub fn submit_effect(&mut self, effect: impl Into<Effect>) {
        self.submit_message(Message::Effect(effect.into()));
    }
}

impl<Intent, Effect> Clone for MessagePort<Intent, Effect> {
    fn clone(&self) -> Self {
        let Self { message_tx } = self;
        let message_tx = message_tx.clone();
        Self { message_tx }
    }
}
