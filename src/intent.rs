// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use crate::Action;

/// Outcome of handling an intent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentHandled<Rejected, Effect, Task> {
    /// Intent has been rejected
    Rejected(Rejected),

    /// Intent has been accepted
    Accepted(Option<Action<Effect, Task>>),
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
    pub fn accepted<E, T>(action: impl Into<Option<Action<E, T>>>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
    {
        Self::Accepted(action.into().map(Action::map_from))
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
            IntentHandled::Accepted(action) => Self::Accepted(action.map(Action::map_from)),
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
