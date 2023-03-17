// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

/// An action is either an effect or a task
///
/// Actions are the result of processing a messages.
///
/// Each message induces at most one _next action_ that is
/// processed in turn. Processing of a message finishes
/// when the sequence of generated next actions terminates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action<Effect, Task> {
    ApplyEffect(Effect),
    DispatchTask(Task),
}

impl<Effect, Task> Action<Effect, Task> {
    pub fn map_from<E, T>(from: Action<E, T>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
    {
        match from {
            Action::ApplyEffect(effect) => Self::ApplyEffect(effect.into()),
            Action::DispatchTask(task) => Self::DispatchTask(task.into()),
        }
    }

    pub fn map_into<E, T>(self) -> Action<E, T>
    where
        E: From<Effect>,
        T: From<Task>,
    {
        Action::map_from(self)
    }
}

impl<Effect, Task> Action<Effect, Task> {
    /// Create a new action that applies an effect.
    #[must_use]
    pub fn apply_effect(effect: impl Into<Effect>) -> Self {
        Self::ApplyEffect(effect.into())
    }

    /// Create a new action that dispatches a task.
    #[must_use]
    pub fn dispatch_task(task: impl Into<Task>) -> Self {
        Self::DispatchTask(task.into())
    }
}
