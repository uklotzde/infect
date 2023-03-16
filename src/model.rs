// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::ops::{Add, AddAssign};

use crate::{EffectApplied, IntentHandled};

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
