// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

use crate::EffectApplied;

/// Outcome of handling an intent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentHandled<Rejected, Task, ModelRenderHint> {
    /// Intent has been rejected
    Rejected(Rejected),

    /// Intent has been accepted by applying an effect
    Accepted(EffectApplied<Task, ModelRenderHint>),
}

impl<Rejected, Task, ModelRenderHint> IntentHandled<Rejected, Task, ModelRenderHint> {
    /// Reject an intent
    pub fn rejected<R>(rejected: R) -> Self
    where
        R: Into<Rejected>,
    {
        Self::Rejected(rejected.into())
    }

    /// Accept an intent
    pub fn accepted<T, M>(effect_applied: EffectApplied<T, M>) -> Self
    where
        T: Into<Task>,
        M: Into<ModelRenderHint>,
    {
        Self::Accepted(EffectApplied::map_from(effect_applied))
    }

    /// Map from a differently parameterized type
    pub fn map_from<R, T, M>(from: IntentHandled<R, T, M>) -> Self
    where
        R: Into<Rejected>,
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
    pub fn map_into<R, T, M>(self) -> IntentHandled<R, T, M>
    where
        R: From<Rejected>,
        T: From<Task>,
        M: From<ModelRenderHint>,
    {
        IntentHandled::map_from(self)
    }
}

impl<Rejected, Task, ModelRenderHint> From<EffectApplied<Task, ModelRenderHint>>
    for IntentHandled<Rejected, Task, ModelRenderHint>
{
    fn from(effect_applied: EffectApplied<Task, ModelRenderHint>) -> Self {
        IntentHandled::Accepted(effect_applied)
    }
}

/// Isomorphic representation of [`IntentHandled`] as a  [`Result`].
///
/// [`IntentHandled`] can be converted seamlessly from and into this result type.
pub type IntentHandledResult<Rejected, Task, ModelRenderHint> =
    Result<EffectApplied<Task, ModelRenderHint>, Rejected>;

impl<Rejected, Task, ModelRenderHint, R, T, M> From<IntentHandledResult<R, T, M>>
    for IntentHandled<Rejected, Task, ModelRenderHint>
where
    R: Into<Rejected>,
    Task: From<T>,
    ModelRenderHint: From<M>,
{
    fn from(res: IntentHandledResult<R, T, M>) -> Self {
        match res {
            Ok(effect_applied) => Self::Accepted(effect_applied.map_into()),
            Err(intent_rejected) => Self::Rejected(intent_rejected.into()),
        }
    }
}

impl<Rejected, Task, ModelRenderHint, R, T, M> From<IntentHandled<R, T, M>>
    for IntentHandledResult<Rejected, Task, ModelRenderHint>
where
    R: Into<Rejected>,
    Task: From<T>,
    ModelRenderHint: From<M>,
{
    fn from(intent_handled: IntentHandled<R, T, M>) -> Self {
        match intent_handled {
            IntentHandled::Accepted(effect_applied) => Ok(effect_applied.map_into()),
            IntentHandled::Rejected(rejected) => Err(rejected.into()),
        }
    }
}
