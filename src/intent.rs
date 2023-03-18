// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use crate::Action;

/// Outcome of handling an intent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentHandled<Intent, Effect, Task> {
    /// Intent has been accepted
    Accepted(Option<Action<Effect, Task>>),

    /// Intent has been rejected
    Rejected(Intent),
}

impl<Intent, Effect, Task> IntentHandled<Intent, Effect, Task> {
    /// Accept an intent
    pub fn accepted<E, T>(action: impl Into<Option<Action<E, T>>>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
    {
        Self::Accepted(action.into().map(Action::map_from))
    }

    /// Reject an intent
    pub fn rejected<I>(intent: I) -> Self
    where
        I: Into<Intent>,
    {
        Self::Rejected(intent.into())
    }

    /// Map from a differently parameterized type
    pub fn map_from<I, E, T>(from: IntentHandled<I, E, T>) -> Self
    where
        I: Into<Intent>,
        E: Into<Effect>,
        T: Into<Task>,
    {
        match from {
            IntentHandled::Accepted(action) => Self::Accepted(action.map(Action::map_from)),
            IntentHandled::Rejected(intent) => Self::Rejected(intent.into()),
        }
    }

    /// Map into a differently parameterized type
    pub fn map_into<I, E, T>(self) -> IntentHandled<I, E, T>
    where
        I: From<Intent>,
        E: From<Effect>,
        T: From<Task>,
    {
        IntentHandled::map_from(self)
    }
}
