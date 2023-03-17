// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use futures::{channel::mpsc, StreamExt as _};

use crate::{
    task::TaskContext, Action, EffectApplied, IntentHandled, Message, Model, ModelChanged,
    RenderModel, TaskExecutor,
};

pub type MessageSender<Intent, Effect> = mpsc::Sender<Message<Intent, Effect>>;
pub type MessageReceiver<Intent, Effect> = mpsc::Receiver<Message<Intent, Effect>>;
pub type MessageChannel<Intent, Effect> = (
    MessageSender<Intent, Effect>,
    MessageReceiver<Intent, Effect>,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextMessageProcessed {
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
) -> NextMessageProcessed
where
    R: RenderModel<Model = M>,
    M: Model + fmt::Debug,
    M::Intent: fmt::Debug,
    M::Effect: fmt::Debug,
    M::Task: fmt::Debug,
    T: TaskExecutor<T, Intent = M::Intent, Effect = M::Effect, Task = M::Task> + Clone,
{
    let mut model_changed = ModelChanged::Unchanged;
    let mut number_of_next_actions = 0;
    let mut number_of_messages_sent = 0;
    let mut number_of_tasks_spawned = 0;
    'process_next_message: loop {
        let effect_applied = match next_message {
            Message::Intent(intent) => {
                let next_action = match model.handle_intent(intent) {
                    IntentHandled::Accepted(next_action) => next_action,
                    IntentHandled::Rejected(intent) => {
                        log::debug!("Discarding rejected intent: {intent:?}");
                        None
                    }
                };
                EffectApplied::unchanged(next_action)
            }
            Message::Effect(effect) => {
                log::debug!("Applying effect: {effect:?}");
                model.apply_effect(effect)
            }
        };
        let EffectApplied {
            model_changed: next_model_changed,
            next_action,
        } = effect_applied;
        model_changed += next_model_changed;
        if let Some(next_action) = next_action {
            // Dispatch next action
            number_of_next_actions += 1;
            match next_action {
                Action::ApplyEffect(effect) => {
                    // Processing immediately continues with the corresponding message
                    // during this turn!
                    next_message = Message::Effect(effect);
                    continue 'process_next_message;
                }
                Action::SpawnTask(task) => {
                    log::debug!("Spawning task: {task:?}");
                    task_context.spawn_task(task);
                    number_of_tasks_spawned += 1;
                }
            }
        }
        if model_changed == ModelChanged::MaybeChanged || number_of_next_actions > 0 {
            log::debug!("Rendering model: {model:?}");
            if let Some(observed_intent) = render_model.render_model(model) {
                log::debug!("Observed intent after rendering model: {observed_intent:?}");
                // The corresponding message is enqueued like any other message, i.e.
                // not processed immediately during this turn!
                let message = Message::Intent(observed_intent);
                task_context.send_message(message);
                number_of_messages_sent += 1;
            }
        }
        break;
    }
    log::debug!("Finished processing of next message: number_of_next_actions = {number_of_next_actions}, number_of_messages_sent = {number_of_messages_sent}, number_of_tasks_spawned = {number_of_tasks_spawned}");
    if number_of_messages_sent + number_of_tasks_spawned > 0 {
        NextMessageProcessed::Progressing
    } else {
        NextMessageProcessed::NoProgress
    }
}

/// Runs the message loop
///
/// Terminates when no progress has been made and all pending tasks have finished.
/// Terminates immediately when the message channel is closed.
pub async fn process_messages<M, R, T>(
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
    T: TaskExecutor<T, Intent = M::Intent, Effect = M::Effect, Task = M::Task> + Clone,
{
    while let Some(next_message) = message_rx.next().await {
        log::debug!("Processing next message: {next_message:?}");
        match process_next_message(task_context, model, render_model, next_message) {
            NextMessageProcessed::Progressing => (),
            NextMessageProcessed::NoProgress => {
                if task_context.all_tasks_finished() {
                    log::debug!("Exiting message loop after all tasks finished");
                    break;
                }
                log::debug!("Continuing message loop until all tasks finished");
            }
        }
    }
}
