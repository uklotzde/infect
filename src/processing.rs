// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use futures::StreamExt as _;

use crate::{
    task::TaskContext, EffectApplied, IntentHandled, Message, MessageReceiver, Model, ModelRender,
    ModelRenderHint, TaskExecutor,
};

/// Outcome of processing a single message
#[derive(Debug, Clone)]
pub enum MessageProcessed<IntentRejected> {
    /// A message with an intent has been rejected
    IntentRejected(IntentRejected),

    /// The system is making progress
    ///
    /// New incoming messages are expected to arrive in the channel after
    /// processing the last message. Otherwise the caller should decide if
    /// continuing the message loop is desired or not.
    ///
    /// If rendering the model resulted in an observed intent the corresponding
    /// message has been submitted and the message channel won't be empty. If a
    /// task has been spawned then this task is expected to submit a message
    /// eventually.
    Progressing,

    /// Not [`Self::Progressing`]
    NoProgress,
}

/// Process a single message
#[must_use]
pub fn process_message<M, R, T>(
    task_context: &mut TaskContext<T, M::Intent, M::Effect>,
    model: &mut M,
    render_model: &mut R,
    mut message: Message<M::Intent, M::Effect>,
) -> MessageProcessed<M::IntentRejected>
where
    M: Model + fmt::Debug,
    M::Intent: fmt::Debug,
    M::IntentRejected: fmt::Debug,
    M::Effect: fmt::Debug,
    M::Task: fmt::Debug,
    R: ModelRender<Model = M>,
    T: TaskExecutor<T, Intent = M::Intent, Effect = M::Effect, Task = M::Task> + Clone,
{
    let mut progressing = false;

    loop {
        let effect_applied = match message {
            Message::Intent(intent) => {
                log::debug!("Handling intent: {intent:?}");
                match model.handle_intent(intent) {
                    IntentHandled::Accepted(effect_applied) => effect_applied,
                    IntentHandled::Rejected(intent_rejected) => {
                        log::debug!("Intent rejected: {intent_rejected:?}");
                        return MessageProcessed::IntentRejected(intent_rejected);
                    }
                }
            }
            Message::Effect(effect) => {
                log::debug!("Applying effect: {effect:?}");
                model.apply_effect(effect)
            }
        };
        let EffectApplied {
            task,
            render_hint,
            next_effect,
        } = effect_applied;
        if let Some(task) = task {
            log::debug!("Spawning task: {task:?}");
            task_context.spawn_task(task);
            progressing = true;
        }

        // Verify that the trait implements the contract as documented.
        debug_assert!(!M::RenderHint::default().should_render_model());
        if render_hint.should_render_model() {
            log::debug!("Rendering model: {model:?}");
            if let Some(observed_intent) = render_model.render_model(model, render_hint) {
                log::debug!("Observed intent after rendering model: {observed_intent:?}");
                // The corresponding message is enqueued like any other message,
                // i.e. not processed immediately during this turn!
                task_context.submit_intent(observed_intent);
                progressing = true;
            }
        }
        if let Some(effect) = next_effect {
            message = Message::Effect(effect);
            // Immediately continue processing the message with the next effect
            // before any other, enqueued messages.
        } else {
            break;
        }
    }

    if progressing {
        MessageProcessed::Progressing
    } else {
        MessageProcessed::NoProgress
    }
}

/// Outcome of consuming multiple messages
///
/// The condition with associated data that stopped consuming messages.
#[derive(Debug, Clone)]
pub enum MessagesConsumed<IntentRejected> {
    /// The last message with an intent has been rejected
    IntentRejected(IntentRejected),

    /// The message channel is closed.
    ChannelClosed,

    /// No progress observed after processing the last message from the channel
    ///
    /// This happens when the channel is empty and no task has been spawned
    /// after processing the last message.
    NoProgress,
}

/// Receive and process messages until one of the stop conditions are
/// encountered
///
/// This `async fn` is _cancellation safe_. The only yield point occurs
/// when receiving the next message from the channel.
#[allow(clippy::manual_let_else)] // false positive?
pub async fn consume_messages<M, R, T>(
    message_rx: &mut MessageReceiver<M::Intent, M::Effect>,
    task_context: &mut TaskContext<T, M::Intent, M::Effect>,
    model: &mut M,
    render_model: &mut R,
) -> MessagesConsumed<M::IntentRejected>
where
    M: Model + fmt::Debug,
    M::Intent: fmt::Debug,
    M::IntentRejected: fmt::Debug,
    M::Effect: fmt::Debug,
    M::Task: fmt::Debug,
    R: ModelRender<Model = M>,
    T: TaskExecutor<T, Intent = M::Intent, Effect = M::Effect, Task = M::Task> + Clone,
{
    let mut next_message: Option<Message<M::Intent, M::Effect>> = None;
    loop {
        let message = if let Some(next_message) = next_message.take() {
            next_message
        } else {
            log::trace!("Awaiting next message");
            let Some(next_message) = message_rx.next().await else {
                log::debug!("Stopping after message channel closed");
                return MessagesConsumed::ChannelClosed;
            };
            next_message
        };
        debug_assert!(next_message.is_none());
        log::debug!("Processing message: {message:?}");
        match process_message(task_context, model, render_model, message) {
            MessageProcessed::IntentRejected(intent_rejected) => {
                log::debug!("Stopping after intent rejected: {intent_rejected:?}");
                return MessagesConsumed::IntentRejected(intent_rejected);
            }
            MessageProcessed::Progressing => {
                // Continue by awaiting the next message that is expected
                // to arrive eventually
            }
            MessageProcessed::NoProgress => {
                next_message = match message_rx.try_next() {
                    Ok(Some(next_message)) => Some(next_message),
                    Ok(None) => {
                        log::debug!(
                            "Stopping after no progress observed and message channel closed"
                        );
                        return MessagesConsumed::ChannelClosed;
                    }
                    Err(_) => {
                        // The message channel is empty but not closed
                        log::debug!(
                            "Stopping after no progress observed and no next message ready"
                        );
                        return MessagesConsumed::NoProgress;
                    }
                };
            }
        }
    }
}
