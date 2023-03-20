// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use futures::StreamExt as _;

use crate::{
    task::TaskContext, EffectApplied, IntentAccepted, IntentHandled, Message, MessageReceiver,
    Model, ModelChanged, ModelRender, TaskExecutor,
};

/// Outcome of processing a single message
#[derive(Debug, Clone)]
pub enum MessageProcessed<IntentRejected> {
    /// A message with an intent has been rejected
    IntentRejected(IntentRejected),

    /// A message with an observed intent has been submitted or
    /// a task has been spawned
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
    message: Message<M::Intent, M::Effect>,
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
    let (effect, task) = match message {
        Message::Intent(intent) => {
            log::debug!("Handling intent: {intent:?}");
            match model.handle_intent(intent) {
                IntentHandled::Accepted(accepted) => {
                    log::debug!("Intent accepted: {accepted:?}");
                    match accepted {
                        IntentAccepted::NoEffect => (None, None),
                        IntentAccepted::ApplyEffect(effect) => (Some(effect), None),
                        IntentAccepted::SpawnTask(task) => (None, Some(task)),
                    }
                }
                IntentHandled::Rejected(intent_rejected) => {
                    log::debug!("Intent rejected: {intent_rejected:?}");
                    return MessageProcessed::IntentRejected(intent_rejected);
                }
            }
        }
        Message::Effect(effect) => (Some(effect), None),
    };
    let model_changed;
    let task = if let Some(effect) = effect {
        debug_assert!(task.is_none());
        log::debug!("Applying effect: {effect:?}");
        let effect_applied = model.apply_effect(effect);
        let EffectApplied {
            model_changed: model_changed_by_effect,
            task,
        } = effect_applied;
        model_changed = model_changed_by_effect;
        task
    } else {
        model_changed = ModelChanged::Unchanged;
        task
    };
    if let Some(task) = task {
        log::debug!("Spawning task: {task:?}");
        task_context.spawn_task(task);
        progressing = true;
    }
    match model_changed {
        ModelChanged::Unchanged => {
            // Skip rendering
        }
        ModelChanged::MaybeChanged => {
            log::debug!("Rendering model: {model:?}");
            if let Some(observed_intent) = render_model.render_model(model) {
                log::debug!("Observed intent after rendering model: {observed_intent:?}");
                // The corresponding message is enqueued like any other message, i.e.
                // not processed immediately during this turn!
                task_context.submit_intent(observed_intent);
                progressing = true;
            }
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

    /// Processing the last message indicated that no progress has been made
    /// and no next message is ready.
    NoProgress,
}

/// Receive and process messages until one of the stop conditions are
/// encountered
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
        if next_message.is_none() {
            next_message = message_rx.next().await;
        }
        let Some(message) = next_message.take() else {
            log::debug!("Stopping after message channel closed");
            return MessagesConsumed::ChannelClosed;
        };
        log::debug!("Processing next message: {message:?}");
        match process_message(task_context, model, render_model, message) {
            MessageProcessed::IntentRejected(intent_rejected) => {
                log::debug!("Stopping after intent rejected: {intent_rejected:?}");
                return MessagesConsumed::IntentRejected(intent_rejected);
            }
            MessageProcessed::Progressing => (),
            MessageProcessed::NoProgress => {
                next_message = match message_rx.try_next() {
                    Ok(Some(message)) => Some(message),
                    Ok(None) => {
                        log::debug!("Stopping after no progress made and message channel closed");
                        return MessagesConsumed::ChannelClosed;
                    }
                    Err(_) => {
                        log::debug!("Stopping after no progress made and no next message ready");
                        return MessagesConsumed::NoProgress;
                    }
                };
            }
        }
    }
}
