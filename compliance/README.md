# compliance

For a parental control app the binding constraints are legal and policy, not technical. This folder is a deliverable, not documentation: the consent flows, the security program, and the store checklists gate every release.

- [legal-notes.md](legal-notes.md): COPPA, GDPR, wiretap/two-party consent, CSAM reporting.
- [store-submission-checklist.md](store-submission-checklist.md): what must be true before any App Store or Play submission.
- [threat-model.md](threat-model.md): security posture and the stalkerware boundary.

## The one rule that drives the rest

Be overt, consented, transparent, data-minimizing, and structurally incapable of covert surveillance. An app that *can* hide is stalkerware regardless of intent. Everything else follows from that.

## MUST / MUST NOT (quick reference)

**MUST:** overt agent plus persistent notification on the child device; child-only targeting; verifiable parental consent (COPPA) plus teen notice (GDPR Art. 8); on-device AI inference; encrypt at rest and in transit; retention limits with auto-delete; a written security program; sanctioned OS APIs only; alert-only output.

**MUST NOT:** any stealth/hide mode on any build; sell or share data; use READ_SMS/Call-Log or Accessibility-for-monitoring on the store builds; intercept two-party communication content; store/cache/transmit suspected CSAM anywhere; build a CSAM classifier or hash matcher; route children's content to a third-party cloud by default; monitor adults; silently and undefeatably block uninstall.
