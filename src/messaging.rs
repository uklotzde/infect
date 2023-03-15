// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::{fmt, future::Future, sync::Arc};

use futures::{channel::mpsc, StreamExt as _};

use crate::{
    action::Action,
    state::{IntentHandled, RenderStateFn, State, StateChanged, StateUpdated},
    Message, TaskDispatcher,
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

#[must_use]
pub fn handle_next_message<S, T>(
    shared_task_dispatcher: &Arc<T>,
    state: &mut S,
    message_tx: &mut MessageSender<S::Intent, S::Effect>,
    mut next_message: Message<S::Intent, S::Effect>,
    render_fn: &mut RenderStateFn<S, S::Intent>,
) -> MessageHandled
where
    S: State + fmt::Debug,
    S::Intent: fmt::Debug + Send + 'static,
    S::Effect: fmt::Debug + Send + 'static,
    S::Task: fmt::Debug + 'static,
    T: TaskDispatcher<Intent = S::Intent, Effect = S::Effect, Task = S::Task>,
{
    let mut state_changed = StateChanged::Unchanged;
    let mut number_of_next_actions = 0;
    let mut number_of_messages_sent = 0;
    let mut number_of_tasks_dispatched = 0;
    'process_next_message: loop {
        let state_updated = match next_message {
            Message::Intent(intent) => {
                let next_action = match state.handle_intent(intent) {
                    IntentHandled::Accepted(next_action) => next_action,
                    IntentHandled::Rejected(intent) => {
                        log::debug!("Discarding {intent:?} rejected by {state:?}");
                        None
                    }
                };
                StateUpdated::unchanged(next_action)
            }
            Message::Effect(effect) => state.update(effect),
        };
        let StateUpdated {
            changed: next_state_changed,
            next_action,
        } = state_updated;
        state_changed += next_state_changed;
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
                    shared_task_dispatcher.dispatch_task(
                        Arc::clone(shared_task_dispatcher),
                        message_tx.clone(),
                        task,
                    );
                    number_of_tasks_dispatched += 1;
                }
            }
        }
        if state_changed == StateChanged::MaybeChanged || number_of_next_actions > 0 {
            log::debug!("Rendering current state: {state:?}");
            if let Some(observation_intent) = render_fn(state) {
                log::debug!("Received intent after observing state: {observation_intent:?}");
                send_message(message_tx, Message::Intent(observation_intent));
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

#[allow(clippy::manual_async_fn)] // Intentional to verify the explicit trait bounds at compile time
pub fn message_loop<S, T>(
    shared_task_dispatcher: Arc<T>,
    (mut message_tx, mut message_rx): MessageChannel<S::Intent, S::Effect>,
    mut state: S,
    mut render_state_fn: Box<RenderStateFn<S, S::Intent>>,
) -> impl Future<Output = S> + Send + 'static
where
    S: State + fmt::Debug + Send + 'static,
    S::Intent: fmt::Debug + Send + 'static,
    S::Effect: fmt::Debug + Send + 'static,
    S::Task: fmt::Debug + Send + 'static,
    T: TaskDispatcher<Intent = S::Intent, Effect = S::Effect, Task = S::Task>
        + Send
        + Sync
        + 'static,
{
    async move {
        log::debug!("Starting message loop");
        while let Some(next_message) = message_rx.next().await {
            match handle_next_message(
                &shared_task_dispatcher,
                &mut state,
                &mut message_tx,
                next_message,
                &mut *render_state_fn,
            ) {
                MessageHandled::Progressing => (),
                MessageHandled::NoProgress => {
                    if shared_task_dispatcher.all_tasks_finished() {
                        break;
                    }
                }
            }
        }
        log::debug!("Terminating message loop");
        state
    }
}
