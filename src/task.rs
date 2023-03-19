// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::{fmt, rc::Rc, sync::Arc};

use crate::{submit_effect, submit_intent, submit_message, Message, MessageSender};

/// Task execution context
#[derive(Debug)]
pub struct TaskContext<TaskExecutor, Intent, Effect> {
    pub task_executor: TaskExecutor,
    pub message_tx: MessageSender<Intent, Effect>,
}

impl<TaskExecutor, Intent, Effect> TaskContext<TaskExecutor, Intent, Effect>
where
    Intent: fmt::Debug,
    Effect: fmt::Debug,
    TaskExecutor: crate::TaskExecutor<TaskExecutor, Intent = Intent, Effect = Effect> + Clone,
{
    /// [`submit_message()`]
    pub fn submit_message(&mut self, message: impl Into<Message<Intent, Effect>>) {
        submit_message(&mut self.message_tx, message);
    }

    /// [`submit_intent()`]
    pub fn submit_intent(&mut self, intent: impl Into<Intent>) {
        submit_intent(&mut self.message_tx, intent);
    }

    /// [`submit_effect()`]
    pub fn submit_effect(&mut self, effect: impl Into<Effect>) {
        submit_effect(&mut self.message_tx, effect);
    }

    /// [`TaskExecutor::spawn_task()`]
    pub fn spawn_task(&self, task: impl Into<TaskExecutor::Task>) {
        let context = self.clone();
        self.task_executor.spawn_task(context, task.into());
    }
}

impl<TaskExecutor, Intent, Effect> Clone for TaskContext<TaskExecutor, Intent, Effect>
where
    TaskExecutor: Clone,
{
    fn clone(&self) -> Self {
        let Self {
            task_executor,
            message_tx,
        } = self;
        Self {
            task_executor: task_executor.clone(),
            message_tx: message_tx.clone(),
        }
    }
}

/// Spawn concurrent tasks
pub trait TaskExecutor<T> {
    type Intent;
    type Effect;
    type Task;

    /// Spawns a task
    ///
    /// The spawned task is executed concurrently, e.g. by spawning
    /// an asynchronous task on some executor.
    ///
    /// Tasks can send messages and spawn new tasks through `context`.
    fn spawn_task(&self, context: TaskContext<T, Self::Intent, Self::Effect>, task: Self::Task);
}

impl<T> TaskExecutor<Rc<T>> for Rc<T>
where
    T: TaskExecutor<Rc<T>>,
{
    type Intent = T::Intent;
    type Effect = T::Effect;
    type Task = T::Task;

    fn spawn_task(&self, context: TaskContext<Self, Self::Intent, Self::Effect>, task: Self::Task) {
        T::spawn_task(self, context, task);
    }
}

impl<T> TaskExecutor<Arc<T>> for Arc<T>
where
    T: TaskExecutor<Arc<T>>,
{
    type Intent = T::Intent;
    type Effect = T::Effect;
    type Task = T::Task;

    fn spawn_task(&self, context: TaskContext<Self, Self::Intent, Self::Effect>, task: Self::Task) {
        T::spawn_task(self, context, task);
    }
}
