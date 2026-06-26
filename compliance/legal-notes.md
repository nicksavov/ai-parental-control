# Legal notes

Not legal advice. This is an engineering summary of the constraints that shape the product. Get counsel before launch, especially before operating any hosted service.

## COPPA (US, 16 CFR Part 312; 2025 amendments, full compliance by 2026-04-22)

- Verifiable parental consent is required before collecting an under-13 child's personal information. The parent is the consenting party, which fits this product.
- Separate consent is required before disclosing child data to a third party unless integral to the service. Routing a child's content to a cloud AI vendor is such a disclosure, which is one reason inference stays on-device.
- A written children's-data security program and data-retention limits are required. No indefinite retention.

## GDPR / GDPR-K (EU, Art. 8)

- Digital-consent age is 13 to 16 depending on member state (for example DE and NL set 16; UK, IE, ES set 13). Below the threshold, a holder of parental responsibility must consent, with reasonable efforts to verify.
- A teen at or above the local age holds their own consent and erasure rights. Covertly monitoring an older minor in the EU is legally risky. Provide a child-readable notice and honor access and erasure.
- Appoint an EU representative for an EU launch.

## Wiretap and two-party consent (US)

- Federal law is one-party consent, but roughly 11 states require all-party consent (CA, DE, FL, IL, MD, MA, MI, MT, NH, PA, WA).
- Intercepting a child's two-way communications implicates these even for a parent. "Vicarious parental consent" is recognized only in a minority of courts. Selling software primarily designed to intercept private communications can itself be a federal crime.
- Therefore: do not intercept or record two-party communication content. Favor DNS/category filtering, time limits, and OS-sanctioned APIs. On-device classification that surfaces an alert (not the content) is far safer than interception.

## CSAM mandatory reporting (18 U.S.C. 2258A; expanded by the 2024 REPORT Act)

The most dangerous area. Read carefully.

- Possessing, copying, or transmitting CSAM is a strict-liability federal crime.
- An app or server that handles user media can become an electronic service provider with a mandatory duty to report apparent CSAM to NCMEC's CyberTipline on actual knowledge, with a 1-year preservation duty. Knowing failure carries large fines.
- MUST NOT store, cache, back up, screenshot, log, forward, or transmit suspected CSAM anywhere, including to a cloud moderation API. Routing a child's image to a cloud classifier could itself be unlawful transmission.
- MUST NOT build a CSAM classifier or hash-matching pipeline. PhotoDNA and the NCMEC hash set are access-restricted and unlicensable for an open project, and Apple's NeuralHash was reverse-engineered and abandoned.
- The safe design: detect generic nudity on-device only, alert the parent with no media attached, prompt the child to delete, and never persist the image. This mirrors Apple's Communication Safety and stays off the CSAM third rail.
- If the project ever operates at scale ingesting user media, that is a separate registered-reporting and legal-counsel project, not a feature.

## Licensing of reused components

- GPL-3.0 forks (for example TimeLimit.io, Habitica) force GPL-3.0 on derivatives.
- AGPL-3.0 (AdGuard Home) triggers source disclosure when offered as a network service.
- Permissive (NanoMDM, Fleet) integrate freely.
- Keep copyleft forks in clearly isolated subtrees.
