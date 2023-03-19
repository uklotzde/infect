// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use futures::StreamExt as _;

use crate::{
    task::TaskContext, Action, EffectApplied, IntentHandled, Message, MessageReceiver, Model,
    ModelChanged, ModelRender, TaskExecutor,
};

/// Outcome of processing a single message
#[derive(Debug, Clone)]
pub enum MessageProcessed<Intent> {
    /// A message with an intent has been rejected
    IntentRejected(Intent),

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
) -> MessageProcessed<M::Intent>
where
    R: ModelRender<Model = M>,
    M: Model + fmt::Debug,
    M::Intent: fmt::Debug,
    M::Effect: fmt::Debug,
    M::Task: fmt::Debug,
    T: TaskExecutor<T, Intent = M::Intent, Effect = M::Effect, Task = M::Task> + Clone,
{
    let mut model_changed = ModelChanged::Unchanged;
    let mut effect_count = 0;
    let mut progressing = false;
    let mut next_action = match message {
        Message::Intent(intent) => match model.handle_intent(intent) {
            IntentHandled::Accepted(next_action) => next_action,
            IntentHandled::Rejected(intent) => {
                return MessageProcessed::IntentRejected(intent);
            }
        },
        Message::Effect(effect) => Some(Action::ApplyEffect(effect)),
    };
    while let Some(action) = next_action.take() {
        match action {
            Action::ApplyEffect(effect) => {
                effect_count += 1;
                log::debug!("Applying effect #{effect_count}: {effect:?}");
                let effect_applied = model.apply_effect(effect);
                let EffectApplied {
                    model_changed: model_changed_by_effect,
                    next_action: new_next_action,
                } = effect_applied;
                model_changed += model_changed_by_effect;
                // Processing continues with the next action
                next_action = new_next_action;
            }
            Action::SpawnTask(task) => {
                log::debug!("Spawning task: {task:?}");
                task_context.spawn_task(task);
                progressing = true;
                // Processing stops after spawning a task
                debug_assert!(next_action.is_none());
            }
        }
    }
    if model_changed == ModelChanged::MaybeChanged {
        log::debug!("Rendering model: {model:?}");
        if let Some(observed_intent) = render_model.render_model(model) {
            log::debug!("Observed intent after rendering model: {observed_intent:?}");
            // The corresponding message is enqueued like any other message, i.e.
            // not processed immediately during this turn!
            task_context.submit_intent(observed_intent);
            progressing = true;
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
pub enum MessagesConsumed<Intent> {
    /// The last message with an intent has been rejected
    IntentRejected(Intent),

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
) -> MessagesConsumed<M::Intent>
where
    M: Model + fmt::Debug,
    M::Intent: fmt::Debug,
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
            MessageProcessed::IntentRejected(intent) => {
                log::debug!("Stopping after intent rejected: {intent:?}");
                return MessagesConsumed::IntentRejected(intent);
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
