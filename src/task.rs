// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::{rc::Rc, sync::Arc};

use crate::MessageSender;

#[derive(Debug)]
pub struct TaskContext<TaskDispatcher, Intent, Effect> {
    pub task_dispatcher: TaskDispatcher,
    pub message_tx: MessageSender<Intent, Effect>,
}

impl<TaskDispatcher, Intent, Effect> Clone for TaskContext<TaskDispatcher, Intent, Effect>
where
    TaskDispatcher: Clone,
{
    fn clone(&self) -> Self {
        let Self {
            task_dispatcher,
            message_tx,
        } = self;
        Self {
            task_dispatcher: task_dispatcher.clone(),
            message_tx: message_tx.clone(),
        }
    }
}

pub trait TaskDispatcher<T> {
    type Intent;
    type Effect;
    type Task;

    /// Keep track of pending tasks
    ///
    /// The message loop only terminates after all tasks have finished.
    #[must_use]
    fn all_tasks_finished(&self) -> bool;

    /// Dispatch a task
    ///
    /// The dispatched task is executed concurrently, e.g. by spawning
    /// an asynchronous task on some executor.
    ///
    /// While running tasks can send messages (`message_tx`) and dispatch
    /// more tasks (`shared_self`).
    ///
    /// The `task_dispatcher` parameter is needed for technical reasons.
    fn dispatch_task(&self, context: TaskContext<T, Self::Intent, Self::Effect>, task: Self::Task);
}

impl<T> TaskDispatcher<Rc<T>> for Rc<T>
where
    T: TaskDispatcher<Rc<T>>,
{
    type Intent = T::Intent;
    type Effect = T::Effect;
    type Task = T::Task;

    fn all_tasks_finished(&self) -> bool {
        T::all_tasks_finished(self)
    }

    fn dispatch_task(
        &self,
        context: TaskContext<Self, Self::Intent, Self::Effect>,
        task: Self::Task,
    ) {
        T::dispatch_task(self, context, task);
    }
}

impl<T> TaskDispatcher<Arc<T>> for Arc<T>
where
    T: TaskDispatcher<Arc<T>>,
{
    type Intent = T::Intent;
    type Effect = T::Effect;
    type Task = T::Task;

    fn all_tasks_finished(&self) -> bool {
        T::all_tasks_finished(self)
    }

    fn dispatch_task(
        &self,
        context: TaskContext<Self, Self::Intent, Self::Effect>,
        task: Self::Task,
    ) {
        T::dispatch_task(self, context, task);
    }
}
