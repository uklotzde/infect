// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use crate::{Action, ModelChanged};

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
    #[must_use]
    pub fn unchanged<E, T>(next_action: Option<Action<E, T>>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
    {
        Self {
            model_changed: ModelChanged::Unchanged,
            next_action: next_action.map(Action::map_from),
        }
    }

    #[must_use]
    pub fn maybe_changed<E, T>(next_action: Option<Action<E, T>>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
    {
        Self {
            model_changed: ModelChanged::MaybeChanged,
            next_action: next_action.map(Action::map_from),
        }
    }

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
}
