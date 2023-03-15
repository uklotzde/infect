// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

/// A message is either an intent or an effect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message<Intent, Effect> {
    Intent(Intent),
    Effect(Effect),
}

impl<Intent, Effect> Message<Intent, Effect> {
    pub fn from_intent(intent: impl Into<Intent>) -> Self {
        Self::Intent(intent.into())
    }

    pub fn from_effect(effect: impl Into<Effect>) -> Self {
        Self::Effect(effect.into())
    }
}
