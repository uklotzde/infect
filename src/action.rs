// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

/// An effect or a task
///
/// Actions are the result of handling intents or applying
/// effects.
///
/// Each intent or effect induces at most one _next action_.
/// Next actions are dispatched immediately before dequeuing
/// the next message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action<Effect, Task> {
    /// Apply an effect
    ApplyEffect(Effect),

    /// Spawn a task
    SpawnTask(Task),
}

impl<Effect, Task> Action<Effect, Task> {
    /// Map from a differently parameterized type
    pub fn map_from<E, T>(from: Action<E, T>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
    {
        match from {
            Action::ApplyEffect(effect) => Self::ApplyEffect(effect.into()),
            Action::SpawnTask(task) => Self::SpawnTask(task.into()),
        }
    }

    /// Map into a differently parameterized type
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

    /// Create a new action that spawns a task.
    #[must_use]
    pub fn spawn_task(task: impl Into<Task>) -> Self {
        Self::SpawnTask(task.into())
    }
}
