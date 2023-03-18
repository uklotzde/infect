// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use futures::StreamExt as _;

use crate::{
    task::TaskContext, Action, EffectApplied, IntentHandled, Message, MessageReceiver, Model,
    ModelChanged, RenderModel, TaskExecutor,
};

#[derive(Debug, Clone)]
pub enum NextMessageProcessed<Intent> {
    IntentRejected(Intent),
    Progressing,
    NoProgress,
}

/// Process the next message
#[must_use]
pub fn process_next_message<M, R, T>(
    task_context: &mut TaskContext<T, M::Intent, M::Effect>,
    model: &mut M,
    render_model: &mut R,
    next_message: Message<M::Intent, M::Effect>,
) -> NextMessageProcessed<M::Intent>
where
    R: RenderModel<Model = M>,
    M: Model + fmt::Debug,
    M::Intent: fmt::Debug,
    M::Effect: fmt::Debug,
    M::Task: fmt::Debug,
    T: TaskExecutor<T, Intent = M::Intent, Effect = M::Effect, Task = M::Task> + Clone,
{
    let mut model_changed = ModelChanged::Unchanged;
    let mut effect_count = 0;
    let mut progressing = false;
    let mut next_action = match next_message {
        Message::Intent(intent) => match model.handle_intent(intent) {
            IntentHandled::Accepted(next_action) => next_action,
            IntentHandled::Rejected(intent) => {
                return NextMessageProcessed::IntentRejected(intent);
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
            let message = Message::Intent(observed_intent);
            task_context.send_message(message);
            progressing = true;
        }
    }
    if progressing {
        NextMessageProcessed::Progressing
    } else {
        NextMessageProcessed::NoProgress
    }
}

/// The condition that stopped message processing.
#[derive(Debug, Clone)]
pub enum ProcessingMessagesStopped<Intent> {
    /// An intent has been rejected
    IntentRejected(Intent),

    /// The message channel is closed.
    ChannelClosed,

    /// Processing the last message indicated that no progress has been made
    /// and no next message is ready.
    NoProgress,
}

/// Process messages until one of the stop conditions occur.
pub async fn process_messages<M, R, T>(
    message_rx: &mut MessageReceiver<M::Intent, M::Effect>,
    task_context: &mut TaskContext<T, M::Intent, M::Effect>,
    model: &mut M,
    render_model: &mut R,
) -> ProcessingMessagesStopped<M::Intent>
where
    M: Model + fmt::Debug,
    M::Intent: fmt::Debug,
    M::Effect: fmt::Debug,
    M::Task: fmt::Debug,
    R: RenderModel<Model = M>,
    T: TaskExecutor<T, Intent = M::Intent, Effect = M::Effect, Task = M::Task> + Clone,
{
    let mut next_message: Option<Message<M::Intent, M::Effect>> = None;
    loop {
        if next_message.is_none() {
            next_message = message_rx.next().await;
        }
        let Some(message) = next_message.take() else {
            log::debug!("Stopping after message channel closed");
            return ProcessingMessagesStopped::ChannelClosed;
        };
        log::debug!("Processing next message: {message:?}");
        match process_next_message(task_context, model, render_model, message) {
            NextMessageProcessed::IntentRejected(intent) => {
                log::debug!("Stopping after intent rejected: {intent:?}");
                return ProcessingMessagesStopped::IntentRejected(intent);
            }
            NextMessageProcessed::Progressing => (),
            NextMessageProcessed::NoProgress => {
                next_message = match message_rx.try_next() {
                    Ok(Some(message)) => Some(message),
                    Ok(None) => {
                        log::debug!("Stopping after no progress made and message channel closed");
                        return ProcessingMessagesStopped::ChannelClosed;
                    }
                    Err(_) => {
                        log::debug!("Stopping after no progress made and no next message ready");
                        return ProcessingMessagesStopped::NoProgress;
                    }
                };
            }
        }
    }
}
