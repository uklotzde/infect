// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use crate::message::MessageSender;

pub trait TaskDispatcher {
    type Intent;
    type Effect;
    type Task;

    fn all_tasks_finished(&self) -> bool;

    fn dispatch_task(
        &self,
        shared_self: Arc<Self>,
        message_tx: MessageSender<Self::Intent, Self::Effect>,
        task: Self::Task,
    );
}