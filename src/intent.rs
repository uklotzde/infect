// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use crate::EffectApplied;

/// Outcome of handling an intent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentHandled<Rejected, Task> {
    /// Intent has been rejected
    Rejected(Rejected),

    /// Intent has been accepted by applying an effect
    Accepted(EffectApplied<Task>),
}

impl<Rejected, Task> IntentHandled<Rejected, Task> {
    /// Reject an intent
    pub fn rejected<R>(rejected: R) -> Self
    where
        R: Into<Rejected>,
    {
        Self::Rejected(rejected.into())
    }

    /// Accept an intent
    pub fn accepted<E, T>(effect_applied: EffectApplied<T>) -> Self
    where
        T: Into<Task>,
    {
        Self::Accepted(EffectApplied::map_from(effect_applied))
    }

    /// Map from a differently parameterized type
    pub fn map_from<R, T>(from: IntentHandled<R, T>) -> Self
    where
        R: Into<Rejected>,
        T: Into<Task>,
    {
        match from {
            IntentHandled::Rejected(rejected) => Self::Rejected(rejected.into()),
            IntentHandled::Accepted(effect_applied) => {
                Self::Accepted(EffectApplied::map_from(effect_applied))
            }
        }
    }

    /// Map into a differently parameterized type
    pub fn map_into<R, T>(self) -> IntentHandled<R, T>
    where
        R: From<Rejected>,
        T: From<Task>,
    {
        IntentHandled::map_from(self)
    }
}

impl<Rejected, Task> From<EffectApplied<Task>> for IntentHandled<Rejected, Task> {
    fn from(effect_applied: EffectApplied<Task>) -> Self {
        IntentHandled::Accepted(effect_applied)
    }
}
