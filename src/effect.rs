// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use crate::ModelChanged;

/// Outcome of applying an effect to the model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectApplied<Effect, Task, ModelRenderHint> {
    /// A hint for rendering the model
    pub render_hint: ModelRenderHint,

    /// A follow-up task for triggering side-effects
    pub task: Option<Task>,

    /// A follow-up effect that will be processed before any queued effects
    ///
    /// Useful for deferring the application of received effects while a
    /// side-effect is pending. When the side-effect finished these deferred
    /// effects could then be recalled one after another before continuing
    /// with the regular message processing.
    pub next_effect: Option<Effect>,
}

impl<Effect, Task, ModelRenderHint> EffectApplied<Effect, Task, ModelRenderHint>
where
    ModelRenderHint: crate::ModelRenderHint,
{
    /// Mark the model as unchanged
    #[must_use]
    pub fn unchanged() -> Self {
        let render_hint = ModelRenderHint::default();
        debug_assert!(!render_hint.should_render_model());
        Self {
            render_hint: Default::default(),
            task: None,
            next_effect: None,
        }
    }

    /// Mark the model as unchanged and dispatch a task
    #[must_use]
    pub fn unchanged_task<T>(task: impl Into<Option<T>>) -> Self
    where
        T: Into<Task>,
    {
        Self {
            task: task.into().map(Into::into),
            ..Self::unchanged()
        }
    }

    /// Mark the model as unchanged and apply a next effect
    #[must_use]
    pub fn unchanged_next<E>(next_effect: impl Into<Option<E>>) -> Self
    where
        E: Into<Effect>,
    {
        Self {
            next_effect: next_effect.into().map(Into::into),
            ..Self::unchanged()
        }
    }
}

impl<Effect, Task> EffectApplied<Effect, Task, ModelChanged> {
    /// Mark the model as maybe changed
    #[must_use]
    pub const fn maybe_changed() -> Self {
        Self {
            render_hint: ModelChanged::MaybeChanged,
            task: None,
            next_effect: None,
        }
    }

    /// Mark the model as maybe changed and dispatch a task
    #[must_use]
    pub fn maybe_changed_task<T>(task: impl Into<Option<T>>) -> Self
    where
        T: Into<Task>,
    {
        Self {
            task: task.into().map(Into::into),
            ..Self::maybe_changed()
        }
    }

    /// Mark the model as maybe changed and apply a next effect
    #[must_use]
    pub fn maybe_changed_next<E>(next_effect: impl Into<Option<E>>) -> Self
    where
        E: Into<Effect>,
    {
        Self {
            next_effect: next_effect.into().map(Into::into),
            ..Self::maybe_changed()
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
        let render_hint = render_hint.into();
        let task = task.map(Into::into);
        let next_effect = next_effect.map(Into::into);
        Self {
            render_hint,
            task,
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
