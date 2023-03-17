// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

/// A message is either an intent or an effect
///
/// In React-terms a `Message` corresponds to an _action_. The distinction
/// into _intents_ and _effects_ allows to distinguish the semantic meaning.
///
/// Intents are supposed to happen (in the future), e.g. when submitting
/// a command. When _rejected_ an intent doesn't result in any effects.
/// When accepted it creates either an immediate effect or side-effects.
/// Side-effects originate from concurrently executed _tasks_. Intents
/// do not alter the current model state.
///
/// Effects on the other hand are inevitable and cannot be ignored. They
/// must be handled by applying them to the current model state with the
/// ability to mutate it. An event is an effect that represents the
/// observation of some undeniable fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message<Intent, Effect> {
    Intent(Intent),
    Effect(Effect),
}
