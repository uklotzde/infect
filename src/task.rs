// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::{fmt, rc::Rc, sync::Arc};

use crate::{send_message, Message, MessageSender};

#[derive(Debug)]
pub struct TaskContext<TaskExecutor, Intent, Effect> {
    task_executor: TaskExecutor,
    message_tx: MessageSender<Intent, Effect>,
}

impl<TaskExecutor, Intent, Effect> TaskContext<TaskExecutor, Intent, Effect>
where
    Intent: fmt::Debug,
    Effect: fmt::Debug,
    TaskExecutor: crate::TaskExecutor<TaskExecutor, Intent = Intent, Effect = Effect> + Clone,
{
    pub fn send_message(&mut self, message: Message<Intent, Effect>) {
        send_message(&mut self.message_tx, message);
    }

    pub fn spawn_task(&self, task: TaskExecutor::Task) {
        let context = self.clone();
        self.task_executor.spawn_task(context, task);
    }

    pub fn all_tasks_finished(&self) -> bool {
        self.task_executor.all_tasks_finished()
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

pub trait TaskExecutor<T> {
    type Intent;
    type Effect;
    type Task;

    /// Keep track of pending tasks
    ///
    /// The message loop only terminates after all tasks have finished.
    #[must_use]
    fn all_tasks_finished(&self) -> bool;

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

    fn all_tasks_finished(&self) -> bool {
        T::all_tasks_finished(self)
    }

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

    fn all_tasks_finished(&self) -> bool {
        T::all_tasks_finished(self)
    }

    fn spawn_task(&self, context: TaskContext<Self, Self::Intent, Self::Effect>, task: Self::Task) {
        T::spawn_task(self, context, task);
    }
}
