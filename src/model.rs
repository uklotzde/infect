// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::ops::{Add, AddAssign};

use crate::{EffectApplied, IntentHandled};

/// Model change indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelChanged {
    /// The model has not changed
    ///
    /// Only return this variant if the model has NOT changed, i.e. if no
    /// no changes are observable and rendering could be skipped. If unsure
    /// or when in doubt return [`Self::MaybeChanged`] to re-render the
    /// model.
    Unchanged,

    /// The model might have changed and needs to be re-rendered
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

/// A stateful model
pub trait Model {
    type Intent;
    type Effect;
    type Task;

    /// Handle an intent
    ///
    /// Intents are comparable to commands. If accepted they might
    /// trigger (immediate) effects or side-effects.
    ///
    /// The model remains unchanged if an intent is rejected and
    /// returned to the caller.
    #[must_use]
    fn handle_intent(
        &self,
        intent: Self::Intent,
    ) -> IntentHandled<Self::Intent, Self::Effect, Self::Task>;

    /// Apply an effect on the model
    ///
    /// Ideally, applying effects is deterministic. In this case recording
    /// and replaying the sequence of effects is sufficient for reconstructing
    /// each intermediate state of the model.
    #[must_use]
    fn apply_effect(&mut self, effect: Self::Effect) -> EffectApplied<Self::Effect, Self::Task>;
}

/// Render the model after changed
pub trait ModelRender {
    type Model: Model;

    /// Render the model after changed
    ///
    /// Might return an observed intent that is enqueued as a message
    /// and handled in turn later.
    #[must_use]
    fn render_model(&mut self, model: &Self::Model) -> Option<<Self::Model as Model>::Intent>;
}
