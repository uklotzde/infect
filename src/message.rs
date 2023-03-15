// SPDX-FileCopyrightText: The infect authors
// SPDX-License-Identifier: MPL-2.0

/// A message is either an intent or an effect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message<Intent, Effect> {
    Intent(Intent),
    Effect(Effect),
}
