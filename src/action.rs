// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action<Effect, Task> {
    ApplyEffect(Effect),
    DispatchTask(Task),
}

impl<Effect, Task> Action<Effect, Task> {
    /// Apply an effect immediately.
    pub fn apply_effect(effect: impl Into<Effect>) -> Self {
        Self::ApplyEffect(effect.into())
    }

    /// Trigger side-effects by dispatching a task.
    pub fn dispatch_task(task: impl Into<Task>) -> Self {
        Self::DispatchTask(task.into())
    }
}
