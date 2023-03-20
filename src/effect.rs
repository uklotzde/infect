// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use crate::ModelChanged;

/// Outcome of applying an effect to the model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectApplied<Task> {
    /// Changes of the model
    pub model_changed: ModelChanged,

    /// The follow-up task
    pub task: Option<Task>,
}

impl<Task> EffectApplied<Task> {
    /// Mark the model as unchanged
    #[must_use]
    pub fn unchanged<T>(task: impl Into<Option<T>>) -> Self
    where
        T: Into<Task>,
    {
        Self {
            model_changed: ModelChanged::Unchanged,
            task: task.into().map(Into::into),
        }
    }

    /// Mark the model as unchanged without a next task
    #[must_use]
    pub const fn unchanged_done() -> Self {
        Self {
            model_changed: ModelChanged::Unchanged,
            task: None,
        }
    }

    /// Mark the model as maybe changed
    #[must_use]
    pub fn maybe_changed<T>(task: impl Into<Option<T>>) -> Self
    where
        T: Into<Task>,
    {
        Self {
            model_changed: ModelChanged::MaybeChanged,
            task: task.into().map(Into::into),
        }
    }

    /// Mark the model as maybe changed without a next task
    #[must_use]
    pub const fn maybe_changed_done() -> Self {
        Self {
            model_changed: ModelChanged::MaybeChanged,
            task: None,
        }
    }

    /// Map from a differently parameterized type
    pub fn map_from<T>(from: EffectApplied<T>) -> Self
    where
        T: Into<Task>,
    {
        let EffectApplied {
            model_changed,
            task,
        } = from;
        let task = task.map(Into::into);
        Self {
            model_changed,
            task,
        }
    }

    /// Map into a differently parameterized type
    pub fn map_into<T>(self) -> EffectApplied<T>
    where
        T: From<Task>,
    {
        EffectApplied::map_from(self)
    }
}
