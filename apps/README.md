# apps

The parent dashboard and the per-OS child agents. Agents are native because every privileged API is OS-specific; the parent side is a shared web core.

| App | What | Stack | Phase |
|---|---|---|---|
| [parent-web](parent-web) | Parent dashboard (also wrapped for mobile/desktop) | React + TypeScript PWA, Capacitor shells | v0 |
| [android-child](android-child) | Child agent (two build flavors: store + sideload deep) | Kotlin | v0 |
| [android-parent](android-parent) | Optional native parent app | Kotlin or Capacitor shell | v1 |
| [windows-agent](windows-agent) | Child agent (Windows service) | C# / .NET 8 | v0 |
| [macos-agent](macos-agent) | Child agent (MDM profile plus helper) | Swift | v3 |
| [ios-child](ios-child) | Child agent | Swift, FamilyControls | v3 |

All agents link the shared Rust core in [../packages](../packages) for pairing, policy, alert construction, and crypto. All child agents must satisfy the overt-only invariant (visible icon plus persistent monitoring notification, no way to hide).
