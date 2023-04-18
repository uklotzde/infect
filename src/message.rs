// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

/// An intent or an effect
///
/// In React-terms a [`Message`] corresponds to an _action_. The distinction
/// into _intents_ and _effects_ allows to distinguish the semantic meaning.
///
/// Intents are supposed to happen (in the future), e.g. when submitting
/// a command. When _rejected_ an intent doesn't result in any effects.
///
/// Effects on the other hand are inevitable and cannot be ignored. Effects
/// must be handled by applying them to the current, mutable model state.
///
/// When accepted/applied both intents and effects create either an immediate
/// effect or side-effects. Side-effects originate from concurrently executed
/// _tasks_. Tasks are supposed emit one or more effects eventually.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message<Intent, Effect> {
    /// An intent
    ///
    /// An intent is a proposal that might be rejected before causing
    /// an effect. After accepted the corresponding effect is applied.
    Intent(Intent),

    /// An effect
    ///
    /// Effects cannot be rejected. But they might have _no effect_, either
    /// if applying them causes no changes or if they are inapplicable and
    /// dropped somehow.
    Effect(Effect),
}
