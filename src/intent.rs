// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

/// Outcome of handling an intent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentHandled<Rejected, Effect, Task> {
    /// Intent has been rejected
    Rejected(Rejected),

    /// Intent has been accepted
    Accepted(IntentAccepted<Effect, Task>),
}

impl<Rejected, Effect, Task> IntentHandled<Rejected, Effect, Task> {
    /// Reject an intent
    pub fn rejected<R>(rejected: R) -> Self
    where
        R: Into<Rejected>,
    {
        Self::Rejected(rejected.into())
    }

    /// Accept an intent
    pub fn accepted<E, T>(accepted: IntentAccepted<E, T>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
    {
        Self::Accepted(IntentAccepted::map_from(accepted))
    }

    /// Map from a differently parameterized type
    pub fn map_from<R, E, T>(from: IntentHandled<R, E, T>) -> Self
    where
        R: Into<Rejected>,
        E: Into<Effect>,
        T: Into<Task>,
    {
        match from {
            IntentHandled::Rejected(rejected) => Self::Rejected(rejected.into()),
            IntentHandled::Accepted(accepted) => Self::Accepted(IntentAccepted::map_from(accepted)),
        }
    }

    /// Map into a differently parameterized type
    pub fn map_into<R, E, T>(self) -> IntentHandled<R, E, T>
    where
        R: From<Rejected>,
        E: From<Effect>,
        T: From<Task>,
    {
        IntentHandled::map_from(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentAccepted<Effect, Task> {
    /// No changes needed
    NoEffect,

    /// Apply an effect
    ApplyEffect(Effect),

    /// Induce side-effects by spawning a task
    SpawnTask(Task),
}

impl<Effect, Task> IntentAccepted<Effect, Task> {
    /// Map from a differently parameterized type
    pub fn map_from<E, T>(from: IntentAccepted<E, T>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
    {
        match from {
            IntentAccepted::NoEffect => Self::NoEffect,
            IntentAccepted::ApplyEffect(effect) => Self::ApplyEffect(effect.into()),
            IntentAccepted::SpawnTask(task) => Self::SpawnTask(task.into()),
        }
    }

    /// Map into a differently parameterized type
    pub fn map_into<E, T>(self) -> IntentAccepted<E, T>
    where
        E: From<Effect>,
        T: From<Task>,
    {
        IntentAccepted::map_from(self)
    }
}

impl<Effect, Task> IntentAccepted<Effect, Task> {
    /// Apply an effect
    #[must_use]
    pub fn apply_effect(effect: impl Into<Effect>) -> Self {
        Self::ApplyEffect(effect.into())
    }

    /// Spawn a task
    #[must_use]
    pub fn spawn_task(task: impl Into<Task>) -> Self {
        Self::SpawnTask(task.into())
    }
}
