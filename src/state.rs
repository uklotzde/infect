// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use std::ops::{Add, AddAssign};

use crate::Action;

/// Perceptible effect when updating the state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateChanged {
    /// The state has not changed
    Unchanged,

    /// The state might have changed
    ///
    /// False positives are allowed, i.e. when unsure or when determining
    /// if the state has actually changed is either costly or impossible
    /// then default to this variant.
    MaybeChanged,
}

impl Add<StateChanged> for StateChanged {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Unchanged, Self::Unchanged) => Self::Unchanged,
            (_, _) => Self::MaybeChanged,
        }
    }
}

impl AddAssign for StateChanged {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateUpdated<Effect, Task> {
    /// The outcome on the state itself
    ///
    /// The state might have been modified during by the update
    /// operation.
    pub changed: StateChanged,

    /// The next action
    ///
    /// Updating the state results in 0 or 1 next action(s).
    pub next_action: Option<Action<Effect, Task>>,
}

impl<Effect, Task> StateUpdated<Effect, Task> {
    #[must_use]
    pub fn unchanged(next_action: impl Into<Option<Action<Effect, Task>>>) -> Self {
        Self {
            changed: StateChanged::Unchanged,
            next_action: next_action.into(),
        }
    }

    #[must_use]
    pub fn maybe_changed(next_action: impl Into<Option<Action<Effect, Task>>>) -> Self {
        Self {
            changed: StateChanged::MaybeChanged,
            next_action: next_action.into(),
        }
    }
}

#[must_use]
pub fn state_updated<E1, T1, E2, T2>(from: StateUpdated<E1, T1>) -> StateUpdated<E2, T2>
where
    E1: Into<E2>,
    T1: Into<T2>,
{
    let StateUpdated {
        changed,
        next_action,
    } = from;
    let next_action = next_action.map(|action| match action {
        Action::ApplyEffect(effect) => Action::apply_effect(effect),
        Action::DispatchTask(task) => Action::dispatch_task(task),
    });
    StateUpdated {
        changed,
        next_action,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentHandled<Intent, Effect, Task> {
    Accepted(Option<Action<Effect, Task>>),
    Rejected(Intent),
}

pub trait State {
    type Intent;
    type Effect;
    type Task;

    #[must_use]
    fn handle_intent(
        &self,
        intent: Self::Intent,
    ) -> IntentHandled<Self::Intent, Self::Effect, Self::Task>;

    #[must_use]
    fn update(&mut self, effect: Self::Effect) -> StateUpdated<Self::Effect, Self::Task>;
}

pub trait RenderState {
    type State: State;

    #[must_use]
    fn render_state(&mut self, state: &Self::State) -> Option<<Self::State as State>::Intent>;
}
