// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use crate::ModelChanged;

/// Outcome of applying an effect to the model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectApplied<Task, ModelRenderHint> {
    /// The follow-up task
    pub task: Option<Task>,

    /// A hint for rendering the model
    pub render_hint: ModelRenderHint,
}

impl<Task> EffectApplied<Task, ModelChanged> {
    /// Mark the model as unchanged
    #[must_use]
    pub fn unchanged<T>(task: impl Into<Option<T>>) -> Self
    where
        T: Into<Task>,
    {
        Self {
            render_hint: ModelChanged::Unchanged,
            task: task.into().map(Into::into),
        }
    }

    /// Mark the model as unchanged without a next task
    #[must_use]
    pub const fn unchanged_done() -> Self {
        Self {
            render_hint: ModelChanged::Unchanged,
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
            render_hint: ModelChanged::MaybeChanged,
            task: task.into().map(Into::into),
        }
    }

    /// Mark the model as maybe changed without a next task
    #[must_use]
    pub const fn maybe_changed_done() -> Self {
        Self {
            render_hint: ModelChanged::MaybeChanged,
            task: None,
        }
    }
}

impl<Task, ModelRenderHint> EffectApplied<Task, ModelRenderHint> {
    /// Map from a differently parameterized type
    pub fn map_from<T, M>(from: EffectApplied<T, M>) -> Self
    where
        T: Into<Task>,
        M: Into<ModelRenderHint>,
    {
        let EffectApplied { task, render_hint } = from;
        let task = task.map(Into::into);
        let render_hint = render_hint.into();
        Self { task, render_hint }
    }

    /// Map into a differently parameterized type
    pub fn map_into<T, M>(self) -> EffectApplied<T, M>
    where
        T: From<Task>,
        M: From<ModelRenderHint>,
    {
        EffectApplied::map_from(self)
    }
}
