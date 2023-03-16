// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::ops::{Add, AddAssign};

use crate::Action;

/// Perceptible effect when updating the model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelChanged {
    /// The model has not changed
    Unchanged,

    /// The model might have changed
    ///
    /// False positives are allowed, i.e. when unsure or when determining
    /// if the model has actually changed is either costly or impossible
    /// then default to this variant.
    MaybeChanged,
}

impl Add<ModelChanged> for ModelChanged {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Unchanged, Self::Unchanged) => Self::Unchanged,
            (_, _) => Self::MaybeChanged,
        }
    }
}

impl AddAssign for ModelChanged {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

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
    pub fn unchanged(next_action: impl Into<Option<Action<Effect, Task>>>) -> Self {
        Self {
            model_changed: ModelChanged::Unchanged,
            next_action: next_action.into(),
        }
    }

    #[must_use]
    pub fn maybe_changed(next_action: impl Into<Option<Action<Effect, Task>>>) -> Self {
        Self {
            model_changed: ModelChanged::MaybeChanged,
            next_action: next_action.into(),
        }
    }
}

#[must_use]
pub fn effect_applied<E1, T1, E2, T2>(from: EffectApplied<E1, T1>) -> EffectApplied<E2, T2>
where
    E1: Into<E2>,
    T1: Into<T2>,
{
    let EffectApplied {
        model_changed,
        next_action,
    } = from;
    let next_action = next_action.map(|action| match action {
        Action::ApplyEffect(effect) => Action::apply_effect(effect),
        Action::DispatchTask(task) => Action::dispatch_task(task),
    });
    EffectApplied {
        model_changed,
        next_action,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentHandled<Intent, Effect, Task> {
    Accepted(Option<Action<Effect, Task>>),
    Rejected(Intent),
}

/// A stateful model
pub trait Model {
    type Intent;
    type Effect;
    type Task;

    #[must_use]
    fn handle_intent(
        &self,
        intent: Self::Intent,
    ) -> IntentHandled<Self::Intent, Self::Effect, Self::Task>;

    #[must_use]
    fn apply_effect(&mut self, effect: Self::Effect) -> EffectApplied<Self::Effect, Self::Task>;
}

/// A model renderer
pub trait RenderModel {
    type Model: Model;

    /// Render the model
    ///
    /// Might return an observed intent that is enqueued as a message
    /// and handled in tuen later.
    #[must_use]
    fn render_model(&mut self, model: &Self::Model) -> Option<<Self::Model as Model>::Intent>;
}
