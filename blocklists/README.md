# blocklists

DNS category lists and SafeSearch rules used by the device-local DNS filter and the optional self-hosted network filter.

- Category lists map the `filtering.blockedCategories` values in [packages/policy-model](../packages/policy-model) to domain sets.
- SafeSearch rules force the safe-search endpoints for major search engines and YouTube Restricted Mode.
- Prefer reusing maintained upstream lists (for example the lists shipped with AdGuard Home / Pi-hole) over hand-curating, and pin versions.

Filtering is DNS/SNI level by design. We do not do TLS interception (wiretap and store risk). Encrypted DNS (DoH/DoT) can bypass plain DNS filtering; the agent detects and handles known DoH endpoints but this is not airtight.
