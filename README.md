<!-- SPDX-FileCopyrightText: The infect authors -->
<!-- SPDX-License-Identifier: MPL-2.0 -->

# infect

[![Crates.io](https://img.shields.io/crates/v/infect.svg)](https://crates.io/crates/infect)
[![Docs.rs](https://docs.rs/infect/badge.svg)](https://docs.rs/infect)
[![Deps.rs](https://deps.rs/repo/github/uklotzde/infect/status.svg)](https://deps.rs/repo/github/uklotzde/infect)
[![Security audit](https://github.com/uklotzde/infect/actions/workflows/security-audit.yaml/badge.svg)](https://github.com/uklotzde/infect/actions/workflows/security-audit.yaml)
[![Continuous integration](https://github.com/uklotzde/infect/actions/workflows/continuous-integration.yaml/badge.svg)](https://github.com/uklotzde/infect/actions/workflows/continuous-integration.yaml)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)

A variation on the [Model-View-Intent (MVI)][mvi] and [State-Action-Model (SAM)][sam] paradigms using

- _intents_ for rejecting or accepting user interaction and commands,
- (immediate) _effects_ for updating the model according to observed events without feedback,
- _tasks_ for inducing concurrent side-_effects_.

[mvi]: https://hannesdorfmann.com/android/model-view-intent/ "Model-View-Intent (MVI)"
[sam]: https://sam.js.org/ "State-Action-Model (SAM)"

## Naming

The semantic distinction of external triggers, inputs, or _events_ (as in _event sourcing_)
into _intents_ and _effects_ is the characteristic difference from existing approaches.
Both stimuli are combined into a _messages_ for transporting and feeding them into the system.

Each _message_ induces at most one _task_ for triggering side-effects.

Combining _**in**tent_, _ef**fec**t_, and _**t**ask_ gives **infect**.

## License

Licensed under the Mozilla Public License 2.0 (MPL-2.0) (see [MPL-2.0.txt](LICENSES/MPL-2.0.txt) or <https://www.mozilla.org/MPL/2.0/>).

Permissions of this copyleft license are conditioned on making available source code of licensed files and modifications of those files under the same license (or in certain cases, one of the GNU licenses). Copyright and license notices must be preserved. Contributors provide an express grant of patent rights. However, a larger work using the licensed work may be distributed under different terms and without source code for files added in the larger work.

### Contribution

Any contribution intentionally submitted for inclusion in the work by you shall be licensed under the Mozilla Public License 2.0 (MPL-2.0).

It is required to add the following header with the corresponding [SPDX short identifier](https://spdx.dev/ids/) to the top of each file:

```rust
// SPDX-License-Identifier: MPL-2.0
```
