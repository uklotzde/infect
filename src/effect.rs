// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use crate::ModelChanged;

/// Outcome of applying an effect to the model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectApplied<Effect, Task, ModelRenderHint> {
    /// A follow-up task for triggering side-effects
    pub task: Option<Task>,

    /// A hint for rendering the model
    pub render_hint: ModelRenderHint,

    /// A follow-up effect that will be processed before any queued effects
    ///
    /// Useful for deferring the application of received effects while a
    /// side-effect is pending. When the side-effect finished these deferred
    /// effects could then be recalled one after another before continuing
    /// with the regular message processing.
    pub next_effect: Option<Effect>,
}

impl<Effect, Task> EffectApplied<Effect, Task, ModelChanged> {
    /// Mark the model as unchanged
    #[must_use]
    pub fn unchanged<T>(task: impl Into<Option<T>>) -> Self
    where
        T: Into<Task>,
    {
        Self {
            task: task.into().map(Into::into),
            render_hint: ModelChanged::Unchanged,
            next_effect: None,
        }
    }

    /// Mark the model as unchanged without a next task
    #[must_use]
    pub const fn unchanged_done() -> Self {
        Self {
            task: None,
            render_hint: ModelChanged::Unchanged,
            next_effect: None,
        }
    }

    /// Mark the model as maybe changed
    #[must_use]
    pub fn maybe_changed<T>(task: impl Into<Option<T>>) -> Self
    where
        T: Into<Task>,
    {
        Self {
            task: task.into().map(Into::into),
            render_hint: ModelChanged::MaybeChanged,
            next_effect: None,
        }
    }

    /// Mark the model as maybe changed without a next task
    #[must_use]
    pub const fn maybe_changed_done() -> Self {
        Self {
            task: None,
            render_hint: ModelChanged::MaybeChanged,
            next_effect: None,
        }
    }
}

impl<Effect, Task, ModelRenderHint> EffectApplied<Effect, Task, ModelRenderHint> {
    /// Map from a differently parameterized type
    pub fn map_from<E, T, M>(from: EffectApplied<E, T, M>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
        M: Into<ModelRenderHint>,
    {
        let EffectApplied {
            render_hint,
            task,
            next_effect,
        } = from;
        let task = task.map(Into::into);
        let render_hint = render_hint.into();
        let next_effect = next_effect.map(Into::into);
        Self {
            task,
            render_hint,
            next_effect,
        }
    }

    /// Map into a differently parameterized type
    pub fn map_into<E, T, M>(self) -> EffectApplied<E, T, M>
    where
        E: From<Effect>,
        T: From<Task>,
        M: From<ModelRenderHint>,
    {
        EffectApplied::map_from(self)
    }
}
