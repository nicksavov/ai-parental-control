# Contributing

Thanks for helping build a safe, open parental control tool. Please read the design rules in the [README](README.md) first. They are hard constraints, not preferences.

## Contributions we will refuse

Because this code could be repurposed for stalking, we will not merge changes that:

- Add any stealth, hidden, disguised-icon, or "invisible agent" mode, on any build (store or sideload).
- Remove or weaken the persistent monitoring notification or the visible app icon on the child device.
- Add a mode that targets a spouse or any adult rather than a child.
- Intercept, record, store, or forward two-party communication content (calls, full message transcripts with third parties).
- Send a child's raw messages or images off the device by default, or to any third-party cloud without explicit, separate, informed parental consent.
- Add a CSAM classifier, perceptual-hash CSAM matcher, or any pipeline that stores or transmits suspected CSAM.
- Persist raw flagged media or message bodies anywhere (device or server).

If you are unsure whether a change crosses one of these lines, open an issue first.

## How to contribute

1. Open an issue describing the change before large work.
2. Keep the shared contracts in `/packages` (alert schema, policy model, pairing protocol) backward compatible, or version them.
3. Add or update tests. The overt-only invariant and the alert "no raw content" invariant must stay covered by tests; a change that breaks them must fail CI.
4. Match the existing code style of the file you are editing. Keep comments minimal and to the point.

## Licensing of contributions

By contributing you agree your work is licensed under GPL-3.0-or-later (clients, agents, shared core) or AGPL-3.0-or-later (backend), matching the directory you touch.
