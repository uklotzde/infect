// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use crate::EffectApplied;

/// Outcome of handling an intent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentHandled<Rejected, Effect, Task, ModelRenderHint> {
    /// Intent has been rejected
    Rejected(Rejected),

    /// Intent has been accepted by applying an effect
    Accepted(EffectApplied<Effect, Task, ModelRenderHint>),
}

impl<Rejected, Effect, Task, ModelRenderHint>
    IntentHandled<Rejected, Effect, Task, ModelRenderHint>
{
    /// Reject an intent
    pub fn rejected<R>(rejected: R) -> Self
    where
        R: Into<Rejected>,
    {
        Self::Rejected(rejected.into())
    }

    /// Accept an intent
    pub fn accepted<E, T, M>(effect_applied: EffectApplied<E, T, M>) -> Self
    where
        E: Into<Effect>,
        T: Into<Task>,
        M: Into<ModelRenderHint>,
    {
        Self::Accepted(EffectApplied::map_from(effect_applied))
    }

    /// Map from a differently parameterized type
    pub fn map_from<R, E, T, M>(from: IntentHandled<R, E, T, M>) -> Self
    where
        R: Into<Rejected>,
        E: Into<Effect>,
        T: Into<Task>,
        M: Into<ModelRenderHint>,
    {
        match from {
            IntentHandled::Rejected(rejected) => Self::Rejected(rejected.into()),
            IntentHandled::Accepted(effect_applied) => {
                Self::Accepted(EffectApplied::map_from(effect_applied))
            }
        }
    }

    /// Map into a differently parameterized type
    pub fn map_into<R, E, T, M>(self) -> IntentHandled<R, E, T, M>
    where
        R: From<Rejected>,
        E: From<Effect>,
        T: From<Task>,
        M: From<ModelRenderHint>,
    {
        IntentHandled::map_from(self)
    }
}

impl<Rejected, Effect, Task, ModelRenderHint> From<EffectApplied<Effect, Task, ModelRenderHint>>
    for IntentHandled<Rejected, Effect, Task, ModelRenderHint>
{
    fn from(effect_applied: EffectApplied<Effect, Task, ModelRenderHint>) -> Self {
        IntentHandled::Accepted(effect_applied)
    }
}

/// Isomorphic representation of [`IntentHandled`] as a  [`Result`].
///
/// [`IntentHandled`] can be converted seamlessly from and into this result type.
pub type IntentHandledResult<Rejected, Effect, Task, ModelRenderHint> =
    Result<EffectApplied<Effect, Task, ModelRenderHint>, Rejected>;

impl<Rejected, Effect, Task, ModelRenderHint, R, E, T, M> From<IntentHandledResult<R, E, T, M>>
    for IntentHandled<Rejected, Effect, Task, ModelRenderHint>
where
    R: Into<Rejected>,
    Effect: From<E>,
    Task: From<T>,
    ModelRenderHint: From<M>,
{
    fn from(res: IntentHandledResult<R, E, T, M>) -> Self {
        match res {
            Ok(effect_applied) => Self::Accepted(effect_applied.map_into()),
            Err(intent_rejected) => Self::Rejected(intent_rejected.into()),
        }
    }
}

impl<Rejected, Effect, Task, ModelRenderHint, R, E, T, M> From<IntentHandled<R, E, T, M>>
    for IntentHandledResult<Rejected, Effect, Task, ModelRenderHint>
where
    R: Into<Rejected>,
    Effect: From<E>,
    Task: From<T>,
    ModelRenderHint: From<M>,
{
    fn from(intent_handled: IntentHandled<R, E, T, M>) -> Self {
        match intent_handled {
            IntentHandled::Accepted(effect_applied) => Ok(effect_applied.map_into()),
            IntentHandled::Rejected(rejected) => Err(rejected.into()),
        }
    }
}
