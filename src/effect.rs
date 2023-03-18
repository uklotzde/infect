// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use crate::{Action, ModelChanged};

/// Outcome of applying an effect to the model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectApplied<Effect, Task> {
    /// The outcome on the model itself
    ///
    /// The model might have been modified during by the update
    /// operation.
    pub model_changed: ModelChanged,

    /// The next action
    ///
    /// Updating the model results in 0 or 1 next action(s).
    pub next_action: Option<Action<Effect, Task>>,
}

impl<Effect, Task> EffectApplied<Effect, Task> {
    /// Mark the model as unchanged.
    #[must_use]
    pub fn unchanged<E, T>(next_action: impl Into<Option<Action<E, T>>>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
    {
        Self {
            model_changed: ModelChanged::Unchanged,
            next_action: next_action.into().map(Action::map_from),
        }
    }

    /// Mark the model as unchanged and terminate the actions sequence.
    #[must_use]
    pub const fn unchanged_done() -> Self {
        Self {
            model_changed: ModelChanged::Unchanged,
            next_action: None,
        }
    }

    /// Mark the model as maybe changed.
    #[must_use]
    pub fn maybe_changed<E, T>(next_action: impl Into<Option<Action<E, T>>>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
    {
        Self {
            model_changed: ModelChanged::MaybeChanged,
            next_action: next_action.into().map(Action::map_from),
        }
    }

    /// Mark the model as maybe changed and terminate the actions sequence.
    #[must_use]
    pub const fn maybe_changed_done() -> Self {
        Self {
            model_changed: ModelChanged::MaybeChanged,
            next_action: None,
        }
    }

    /// Map from a differently parameterized type
    pub fn map_from<E, T>(from: EffectApplied<E, T>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
    {
        let EffectApplied {
            model_changed,
            next_action,
        } = from;
        let next_action = next_action.map(Action::map_from);
        Self {
            model_changed,
            next_action,
        }
    }

    /// Map into a differently parameterized type
    pub fn map_into<E, T>(self) -> EffectApplied<E, T>
    where
        E: From<Effect>,
        T: From<Task>,
    {
        EffectApplied::map_from(self)
    }
}
