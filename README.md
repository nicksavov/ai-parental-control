# ai-parental-control

A free, open-source, AI-assisted parental control app for the latest Windows 11, macOS, Android, iPadOS, and iPhone. Similar in spirit to Bark and Adora, but on-device, privacy-first, and self-hostable.

> Status: early scaffold. See [docs/architecture.md](docs/architecture.md) for the full design and [docs/roadmap.md](docs/roadmap.md) for the build phases.

## What it does

- Web and content filtering (DNS, SafeSearch, category blocklists), including a self-hosted network filter that needs no agent on the device.
- Screen time reporting, daily limits, bedtime and schedule windows, per-app limits.
- On-device AI content safety: nudity detection in images and harmful-text detection (bullying, self-harm, grooming, drugs, violence). Only alerts leave the device, never the content.
- Location and check-ins.
- A parent dashboard (web app, plus mobile and desktop shells).

## Design rules (non-negotiable)

This project is a parental safety tool, not surveillance software. The same code could be abused for stalking, so these rules are enforced in code and tests, not just policy:

1. **Overt, never covert.** The agent on the child device always shows a visible icon and a persistent monitoring notification. There is no build flag, debug mode, or setting that hides it.
2. **Children only.** No mode targets a spouse or any other adult, ever.
3. **On-device AI, alert-only output.** The backend never sees a message, image, or any AI input. Only structured alerts (`category`, `severity`, `timestamp`, `app source`, optional short snippet) leave the device, end-to-end encrypted from the child device to the parent.
4. **Never store or transmit flagged media.** Content is analyzed in memory and discarded. No cloud routing of a child's content by default.
5. **No CSAM detection pipeline.** We detect generic nudity on-device and alert the parent. We do not build a CSAM classifier or hash matcher (those databases are unlicensable for an open project, and handling such content is a strict-liability crime).
6. **No two-party communication interception.** We favor DNS/category filtering, time limits, and OS-sanctioned APIs over reading call or message bodies.
7. **Tamper is an alert, not a hidden block.** If a child disables or uninstalls the agent, the parent is notified. We do not ship an invisible, unkillable process.

See [CONTRIBUTING.md](CONTRIBUTING.md) for what contributions are refused on these grounds, and [compliance/](compliance/) for the legal basis (COPPA, GDPR, wiretap law, store policy).

## Architecture in one line

Smart edge, thin cloud: fat native agents do all sensing, enforcement, and AI on the device; a thin Go backend only handles pairing, auth, and relays end-to-end-encrypted alert envelopes. The backend can run on a $5 VPS or a home Raspberry Pi, or you can use a hosted instance.

## Platform reality

Platform capability is very uneven, so the roadmap ships where it is feasible first:

| | Windows 11 | Android | iOS / iPadOS | macOS |
|---|---|---|---|---|
| Filtering, screen time, image AI | yes | yes | later (Apple entitlement) | later |
| Full text monitoring | partial | store: previews only; sideload: full | not possible (no API) | partial |

Android ships in two builds: a store-compliant build for reach, and an optional sideloaded/F-Droid "deep" build that adds full on-screen text analysis (and optionally SMS). Both are overt.

## Repository layout

```
/apps        parent dashboard (PWA) and per-OS child agents
/backend     Go coordination service (AGPL-3.0)
/packages    shared Rust core: pairing, policy model, alert schema, proto
/ai          on-device text and image pipelines plus bundled models
/blocklists  DNS category lists and SafeSearch rules
/compliance  privacy, COPPA/GDPR, store checklists, threat model
/infra       self-host (docker compose), CI, sideload updater
/docs        architecture, roadmap, onboarding, ADRs
```

## Licensing

- Clients, agents, and shared core: **GPL-3.0-or-later** ([LICENSE](LICENSE)).
- Backend: **AGPL-3.0-or-later** ([backend/LICENSE](backend/LICENSE)), so any hosted modified backend must publish its source. This prevents a closed-source surveillance service from being spun off the relay.

Bundled AI models keep their own open licenses; see [ai/models/README.md](ai/models/README.md).

## Contact

- Child safety contact: see [SECURITY.md](SECURITY.md).
- Security and vulnerability disclosure: see [SECURITY.md](SECURITY.md).
