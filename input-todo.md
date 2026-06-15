# Input Control TODO

Target: practical 90/10 support for HTML-like input behavior, starting with controls that are already mostly implemented in the engine.

## Planned controls

1. Textual inputs
   - `type="text"` — default
   - `type="password"` for masked display
2. Numeric inputs
   - `type="number"` with `min`, `max`, `step`
   - number entry rejects non-numeric glyphs and clamps to range
3. Range input
   - `type="range"` with `min`, `max`, `step`
   - arrow/page-key stepping and `Home`/`End` extremes
4. Boolean inputs
   - `type="checkbox"` with `checked` and `name`
   - `type="radio"` with `checked`, plus same-`name` grouping

## Associated properties

The following attributes are wired to native input state:

- `value`
- `type`
- `placeholder` (text-like types)
- `checked`
- `name`
- `min`
- `max`
- `step`
- `readonly`
- `disabled`

## Event payloads

- `onInput` carries `event.value` for all text-like inputs.
- `onInput` carries `event.checked` for `checkbox` and `radio`.
- `selectionStart` / `selectionEnd` are exposed for text-like inputs.

## Current status

- Implemented end-to-end in core input state + key handling + bridge propagation.
- Added to the kitchen sink demo with sample controls for:
  - text, number, range, password, checkbox, and radio.
