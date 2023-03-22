// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::{fmt, rc::Rc, sync::Arc};

use crate::{Message, MessagePort};

/// Task execution context
#[derive(Debug)]
pub struct TaskContext<TaskExecutor, Intent, Effect> {
    /// A task executor for spawning sub-tasks
    pub task_executor: TaskExecutor,

    /// A message port for submitting the task's side-effect
    pub message_port: MessagePort<Intent, Effect>,
}

impl<TaskExecutor, Intent, Effect> TaskContext<TaskExecutor, Intent, Effect>
where
    Intent: fmt::Debug,
    Effect: fmt::Debug,
    TaskExecutor: crate::TaskExecutor<TaskExecutor, Intent = Intent, Effect = Effect> + Clone,
{
    /// [`MessagePort::submit_message()`]
    pub fn submit_message(&mut self, message: impl Into<Message<Intent, Effect>>) {
        self.message_port.submit_message(message);
    }

    /// [`MessagePort::submit_intent()`]
    pub fn submit_intent(&mut self, intent: impl Into<Intent>) {
        self.message_port.submit_intent(intent);
    }

    /// [`MessagePort::submit_effect()`]
    pub fn submit_effect(&mut self, effect: impl Into<Effect>) {
        self.message_port.submit_effect(effect);
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
            message_port,
        } = self;
        Self {
            task_executor: task_executor.clone(),
            message_port: message_port.clone(),
        }
    }
}

/// Spawn concurrent tasks
pub trait TaskExecutor<T> {
    /// The intent type
    type Intent;

    /// The effect type
    type Effect;

    /// The task type
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
