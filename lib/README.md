This directory contains multiple modules for composing and decomposing Hangul text.

### Modules
- `jamo` – Modern jamo constants, helpers to create/decompose composite jamo, and converters from compatibility jamo to the modern code points. Also provides the `Character` classifier and the `Jamo` enum used across the crate.
- `block` – `HangulBlock` (initial, vowel, optional final) plus a `BlockComposer` state machine that only accepts valid jamo sequences. Includes helpers to convert blocks into Unicode syllable chars and back.
- `word` – `HangulWordComposer` wraps a `BlockComposer` and a list of completed blocks, letting you stream jamo in, pop them out, and automatically start new syllable blocks when needed.
- `string` – `StringComposer` layers on top of `HangulWordComposer` so you can interleave Hangul input with non-Hangul text; invalid/compatibility jamo are normalized and non-Hangul is passed through.

For more documentation, see the README in the root directory.