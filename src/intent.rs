// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use crate::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentHandled<Intent, Effect, Task> {
    Accepted(Option<Action<Effect, Task>>),
    Rejected(Intent),
}

impl<Intent, Effect, Task> IntentHandled<Intent, Effect, Task> {
    pub fn accepted<E, T>(action: impl Into<Option<Action<E, T>>>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
    {
        Self::Accepted(action.into().map(Action::map_from))
    }

    pub fn rejected<I>(intent: I) -> Self
    where
        I: Into<Intent>,
    {
        Self::Rejected(intent.into())
    }

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
}
