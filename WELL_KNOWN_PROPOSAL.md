# .well-known URI Support Proposal

## Overview
Expanding robotxt to check relevant `.well-known` URIs that provide licensing and scraping guidance.

## Priority 1: Text and Data Mining (TDM)

### `/.well-known/tdmrep.json`
**Status:** Highly relevant for scraping/licensing
**IANA Registered:** Yes
**Reference:** [W3C TDM Reservation Protocol](https://www.w3.org/community/tdmrep/)

**Purpose:** Domain-wide Text and Data Mining (TDM) reservation policies. This complements robots.txt by specifying:
- What you CAN DO with scraped data (not just what you can scrape)
- Commercial vs non-commercial TDM rights
- Licensing requirements for AI training

**Example tdmrep.json:**
```json
[
  {
    "location": "/",
    "tdm-reservation": 1,
    "tdm-policy": "https://example.com/tdm-policy.html"
  },
  {
    "location": "/public/*",
    "tdm-reservation": 0
  }
]
```

**Format:** Array of rules (W3C TDMRep Final Spec 2024-05-10)
- `location` (required): Path pattern with `*` wildcard, `$` end marker
- `tdm-reservation` (required): `1` = reserved (restricted), `0` = unreserved (allowed)
- `tdm-policy` (optional): URL to human-readable policy document

**Matching:** Case-sensitive, most specific match wins (first in array), follows robots.txt pattern conventions

**Integration:**
- Add `tdm_policy: Option<TdmPolicy>` to AnalysisResult
- `TdmPolicy` contains matched rule with reservation status and policy URL

---

## Priority 2: Security and Contact Info

### `/.well-known/security.txt`
**Status:** Useful for responsible scraping
**IANA Registered:** 2018-08-20
**RFC:** [RFC 9116](https://www.rfc-editor.org/rfc/rfc9116.html)

**Purpose:** Security policy and contact information. Useful for:
- Finding who to contact about scraping questions
- Reporting security issues discovered during crawling
- Understanding security preferences

**Example security.txt:**
```
Contact: security@example.com
Expires: 2026-12-31T23:59:59z
Preferred-Languages: en
Policy: https://example.com/security-policy
```

**Integration:**
- Add `security_contact: Option<String>`
- Add `security_policy: Option<String>`

---

## Priority 3: Privacy Controls

### `/.well-known/dnt` and `/.well-known/dnt-policy.txt`
**Status:** Privacy-relevant
**IANA Registered:** 2015-08-19

**Purpose:** Site-wide Do Not Track (DNT) tracking status and policy.

**Integration:**
- Add `dnt_policy: Option<String>`
- Add `respects_dnt: Option<bool>`

### `/.well-known/gpc.json`
**Status:** Privacy-relevant
**Purpose:** Global Privacy Control (GPC) support declaration

**Integration:**
- Add `gpc_enabled: Option<bool>`

---

## Implementation Plan

### Phase 1: TDM Support (Highest Value)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdmRule {
    pub location: String,            // Path pattern (e.g., "/", "/docs/*")
    #[serde(rename = "tdm-reservation")]
    pub tdm_reservation: u8,         // 0 = unreserved, 1 = reserved
    #[serde(rename = "tdm-policy")]
    pub tdm_policy: Option<String>,  // Optional policy URL
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdmPolicy {
    pub rules: Vec<TdmRule>,         // All rules from tdmrep.json
    pub matched_rule: Option<TdmRule>, // Rule that matched the requested path
    pub is_reserved: bool,           // Evaluated: true = TDM restricted
}

// Add to AnalysisResult
pub struct AnalysisResult {
    // ... existing fields ...
    pub tdm_policy: Option<TdmPolicy>,
}
```

### Phase 2: Security and Privacy
```rust
// Add to AnalysisResult
pub struct AnalysisResult {
    // ... existing fields ...
    pub security_contact: Option<String>,
    pub security_policy_url: Option<String>,
    pub dnt_enabled: Option<bool>,
    pub gpc_enabled: Option<bool>,
}
```

### Phase 3: Other Relevant URIs
- `/.well-known/ai-plugin.json` - ChatGPT plugin manifests (AI scraping context)
- `/.well-known/openid-configuration` - Authentication endpoints (for gated content)

---

## Example Enhanced Output

```
================================================================================
URL: https://example.com
Robots.txt: https://example.com/robots.txt
Status: ✓ Success

RSL Licenses (Active):
  📜 https://example.com/license.xml

TDM Policy:
  ⚠️  TDM Reservation: YES (reserved for this path)
  📄 Policy: https://example.com/tdm-policy.html
  🎯 Matched Rule: "/" (tdm-reservation: 1)

Security:
  📧 Contact: security@example.com
  📜 Policy: https://example.com/security-policy

Privacy:
  🔒 Do Not Track: Enabled
  🔒 Global Privacy Control: Enabled
================================================================================
```

---

## CLI Changes

Add new flag:
```bash
robotxt analyze --url https://example.com --check-well-known
```

Or specific checks:
```bash
robotxt analyze --url https://example.com --check-tdm --check-security
```

---

## API Response Changes

```json
{
  "url": "https://example.com",
  "robots_url": "https://example.com/robots.txt",
  "active_licenses": ["https://example.com/license.xml"],
  "tdm_policy": {
    "is_reserved": true,
    "matched_rule": {
      "location": "/",
      "tdm-reservation": 1,
      "tdm-policy": "https://example.com/tdm-policy.html"
    },
    "rules": [
      {
        "location": "/",
        "tdm-reservation": 1,
        "tdm-policy": "https://example.com/tdm-policy.html"
      }
    ]
  },
  "security_contact": "security@example.com",
  "dnt_enabled": true,
  "gpc_enabled": true
}
```

---

## Benefits

1. **Comprehensive Compliance**: Check robots.txt, RSL licenses, and TDM policies in one tool
2. **Legal Protection**: Understand licensing requirements before scraping
3. **Responsible Scraping**: Respect privacy preferences (DNT, GPC)
4. **AI Training Clarity**: Know if content can be used for AI/ML training
5. **Contact Discovery**: Find the right people to ask about scraping permissions

---

## Recommended Implementation Order

1. ✅ **RSL License Support** (Already implemented!)
2. 🔥 **TDM Policy Support** (`tdmrep.json`) - Highest value for scraping use cases
3. 📧 **Security Contact** (`security.txt`) - Low complexity, high utility
4. 🔒 **Privacy Controls** (`dnt`, `gpc.json`) - Important for compliance
5. 🤖 **AI Plugin Manifest** (`ai-plugin.json`) - Future-proofing for AI agents

---

## References

- [IANA Well-Known URIs Registry](https://www.iana.org/assignments/well-known-uris/well-known-uris.xhtml)
- [W3C TDM Reservation Protocol](https://www.w3.org/community/tdmrep/)
- [RFC 9116 - security.txt](https://www.rfc-editor.org/rfc/rfc9116.html)
- [RSL Standard](https://rslstandard.org/rsl)
- [Global Privacy Control](https://globalprivacycontrol.org/)
