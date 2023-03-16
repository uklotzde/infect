// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use futures::{channel::mpsc, StreamExt as _};

use crate::{
    task::TaskContext, Action, EffectApplied, IntentHandled, Message, Model, ModelChanged,
    RenderModel, TaskDispatcher,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageHandled {
    Progressing,
    NoProgress,
}

/// Process the next message
#[must_use]
pub fn process_next_message<M, R, T>(
    task_context: &mut TaskContext<T, M::Intent, M::Effect>,
    model: &mut M,
    render_model: &mut R,
    mut next_message: Message<M::Intent, M::Effect>,
) -> MessageHandled
where
    R: RenderModel<Model = M>,
    M: Model + fmt::Debug,
    M::Intent: fmt::Debug,
    M::Effect: fmt::Debug,
    M::Task: fmt::Debug,
    T: TaskDispatcher<T, Intent = M::Intent, Effect = M::Effect, Task = M::Task> + Clone,
{
    let mut model_changed = ModelChanged::Unchanged;
    let mut number_of_next_actions = 0;
    let mut number_of_messages_sent = 0;
    let mut number_of_tasks_dispatched = 0;
    'process_next_message: loop {
        let effect_applied = match next_message {
            Message::Intent(intent) => {
                let next_action = match model.handle_intent(intent) {
                    IntentHandled::Accepted(next_action) => next_action,
                    IntentHandled::Rejected(intent) => {
                        log::debug!("Discarding {intent:?} rejected by {model:?}");
                        None
                    }
                };
                EffectApplied::unchanged(next_action)
            }
            Message::Effect(effect) => model.apply_effect(effect),
        };
        let EffectApplied {
            model_changed: next_model_changed,
            next_action,
        } = effect_applied;
        model_changed += next_model_changed;
        if let Some(next_action) = next_action {
            number_of_next_actions += 1;
            match next_action {
                Action::ApplyEffect(effect) => {
                    log::debug!("Applying subsequent effect immediately: {effect:?}");
                    next_message = Message::Effect(effect);
                    continue 'process_next_message;
                }
                Action::DispatchTask(task) => {
                    log::debug!("Dispatching task asynchronously: {task:?}");
                    task_context
                        .task_dispatcher
                        .dispatch_task(task_context.clone(), task);
                    number_of_tasks_dispatched += 1;
                }
            }
        }
        if model_changed == ModelChanged::MaybeChanged || number_of_next_actions > 0 {
            log::debug!("Rendering current model: {model:?}");
            if let Some(observation_intent) = render_model.render_model(model) {
                log::debug!("Received intent after observing model: {observation_intent:?}");
                send_message(
                    &mut task_context.message_tx,
                    Message::Intent(observation_intent),
                );
                number_of_messages_sent += 1;
            }
        }
        break;
    }
    log::debug!("number_of_next_actions = {number_of_next_actions}, number_of_messages_sent = {number_of_messages_sent}, number_of_tasks_dispatched = {number_of_tasks_dispatched}");
    if number_of_messages_sent + number_of_tasks_dispatched > 0 {
        MessageHandled::Progressing
    } else {
        MessageHandled::NoProgress
    }
}

/// Runs the message loop
///
/// Terminates when no progress has been made and all pending tasks have finished.
/// Terminates immediately when the message channel is closed.
pub async fn message_loop<M, R, T>(
    message_rx: &mut MessageReceiver<M::Intent, M::Effect>,
    task_context: &mut TaskContext<T, M::Intent, M::Effect>,
    model: &mut M,
    render_model: &mut R,
) where
    M: Model + fmt::Debug,
    M::Intent: fmt::Debug,
    M::Effect: fmt::Debug,
    M::Task: fmt::Debug,
    R: RenderModel<Model = M>,
    T: TaskDispatcher<T, Intent = M::Intent, Effect = M::Effect, Task = M::Task> + Clone,
{
    log::debug!("Starting message loop");
    while let Some(next_message) = message_rx.next().await {
        log::debug!("Processing next message: {next_message:?}");
        match process_next_message(task_context, model, render_model, next_message) {
            MessageHandled::Progressing => (),
            MessageHandled::NoProgress => {
                if task_context.task_dispatcher.all_tasks_finished() {
                    break;
                }
            }
        }
    }
    log::debug!("Terminating message loop");
}
