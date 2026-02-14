# PolicyCheck in the OpenAttribution Ecosystem

## The OpenAttribution Initiative

**Mission**: Enable responsible AI development through clear content licensing and attribution standards.

OpenAttribution addresses the critical gap in AI development: the lack of transparent mechanisms for tracking and attributing content used by AI systems.

## The Three-Part Solution

### 1. **Publisher-Side Standards** (What content owners declare)

**PolicyCheck** checks these publisher-declared policies:

- ✅ **robots.txt** (RFC 9309) - What paths can be crawled
- ✅ **RSL Licenses** - Licensing terms for content use
- 🚧 **TDMRep** (Text & Data Mining) - AI training permissions
- 🚧 **security.txt** (RFC 9116) - Security contacts and policies
- 🚧 **Privacy controls** (DNT, GPC) - Privacy preferences

**PolicyCheck's Role**: Pre-flight compliance checker. Before an AI agent accesses content, it uses PolicyCheck to verify:
- "Can I crawl this URL?"
- "What license terms apply?"
- "Can I use this for AI training?"
- "Who should I contact about access?"

### 2. **Agent-Side Standards** (What AI systems declare)

**AIMS** (AI Manifest Standard) provides:

- **Training Data Provenance**: What data trained this model? What licenses were used?
- **Runtime Content Access**: What content partnerships does this agent have?
- **Agent Identity**: DID-based cryptographic verification
- **Agent-to-Agent Trust**: How agents verify each other before sharing information

**AIMS's Role**: Agents declare their identity and licensing compliance through verifiable manifests.

### 3. **Tracking & Attribution** (What actually happened)

**Telemetry** (Content Attribution Telemetry) tracks:

- **Content Retrieval**: Which content was accessed during a session
- **Content Citation**: Which content was used in responses
- **User Outcomes**: What actions resulted (purchases, signups, etc.)
- **Agent-to-Agent Sessions**: How content flows between AI systems

**Telemetry's Role**: Creates auditable records linking content usage to outcomes, enabling fair attribution and compensation.

## How They Work Together

### Example: Responsible AI Agent Accessing Content

```
┌─────────────────────────────────────────────────────────────────┐
│ Step 1: Pre-Flight Check (PolicyCheck)                            │
│                                                                 │
│ AI Agent wants to access https://example.com/article           │
│   ↓                                                             │
│ PolicyCheck checks:                                               │
│   ✓ robots.txt: Path allowed for "MyBot"                      │
│   ✓ RSL License: https://example.com/license.xml              │
│   ✓ TDMRep: AI training allowed for non-commercial research   │
│   ✓ security.txt: Contact security@example.com for questions  │
│                                                                 │
│ Result: Access allowed with license requirements               │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 2: Agent Identity Verification (AIMS)                     │
│                                                                 │
│ AI Agent presents manifest:                                    │
│   - DID: did:web:mycompany.com:agents:research-bot            │
│   - Training Data: Licensed under XYZ, RSL-compliant          │
│   - Content Access Rights: Research partnership with ABC      │
│   - Cryptographic Signature: Verifiable credentials           │
│                                                                 │
│ Publisher can verify: "This is a legitimate research agent"    │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 3: Content Access & Usage (Telemetry)                     │
│                                                                 │
│ Session: session_abc123                                        │
│   Event: content_retrieved                                     │
│     - content_id: example.com/article                          │
│     - timestamp: 2026-02-14T12:00:00Z                         │
│   Event: content_cited                                         │
│     - content_id: example.com/article                          │
│     - citation_type: quote                                     │
│   Event: outcome                                               │
│     - type: user_engaged                                       │
│     - value: clicked_source_link                               │
│                                                                 │
│ Result: Auditable attribution record                           │
└─────────────────────────────────────────────────────────────────┘
```

## Key Integrations

### PolicyCheck → AIMS Integration

When an agent checks compliance with PolicyCheck, it can:
1. Fetch the RSL license URL
2. Include that license in its AIMS manifest under "Content Access Rights"
3. Prove it checked licensing before access

### PolicyCheck → Telemetry Integration

PolicyCheck's compliance check results can inform telemetry:
1. `content_scope` in session metadata (what licenses apply)
2. `content_source` tracking (where content came from)
3. License compliance flags in event metadata

### AIMS → Telemetry Integration

Telemetry can reference AIMS manifests:
1. Agent identity (DID) in session metadata
2. Training data provenance in attribution calculations
3. License boundaries when content is shared between agents

## Standards Compliance Matrix

| Standard | Type | Status | Handled By |
|----------|------|--------|------------|
| **RFC 9309** (Robots.txt) | Publisher-side | ✅ Implemented | PolicyCheck |
| **RSL** (Responsible Sourcing License) | Publisher-side | ✅ Implemented | PolicyCheck |
| **TDMRep** (Text & Data Mining) | Publisher-side | 🚧 Planned | PolicyCheck |
| **RFC 9116** (security.txt) | Publisher-side | 🚧 Planned | PolicyCheck |
| **W3C DIDs** | Agent identity | ✅ Implemented | AIMS |
| **Verifiable Credentials** | Agent identity | ✅ Implemented | AIMS |
| **A2A Protocol** | Agent communication | ✅ Integrated | AIMS |
| **OpenAttribution Telemetry** | Attribution tracking | ✅ Implemented | Telemetry |

## Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Content Publisher                           │
│                                                                 │
│  Publishes:                                                     │
│    • robots.txt (access rules)                                 │
│    • RSL license (licensing terms)                             │
│    • TDMRep (AI training policy)                               │
│    • security.txt (contact info)                               │
└─────────────────────────────────────────────────────────────────┘
                              ↑
                              │ checked by
                              │
┌─────────────────────────────────────────────────────────────────┐
│                        PolicyCheck                                 │
│                                                                 │
│  Fetches and analyzes publisher policies                       │
│  Returns compliance report to agent                             │
└─────────────────────────────────────────────────────────────────┘
                              ↓
                   compliance report used by
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                       AI Agent                                  │
│                                                                 │
│  • Checks compliance (PolicyCheck)                                │
│  • Presents identity (AIMS manifest)                           │
│  • Tracks usage (Telemetry events)                             │
│  • Respects licensing terms                                    │
└─────────────────────────────────────────────────────────────────┘
                              ↓
                     generates events to
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    Telemetry System                             │
│                                                                 │
│  Receives attribution events:                                  │
│    • content_retrieved                                         │
│    • content_cited                                             │
│    • outcome                                                    │
│  Links to AIMS manifest for full context                       │
└─────────────────────────────────────────────────────────────────┘
                              ↓
                     analyzed by
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                Attribution Consumer                             │
│                                                                 │
│  • Calculates content value/impact                             │
│  • Informs compensation arrangements                           │
│  • Provides publisher dashboards                               │
└─────────────────────────────────────────────────────────────────┘
```

## Use Cases

### Use Case 1: Research AI Complying with Publisher Policies

```python
# Step 1: Check compliance
import policycheck
result = policycheck.analyze("https://publisher.com/article")

if result.active_licenses:
    # Step 2: Fetch license, verify terms
    license = fetch_license(result.active_licenses[0])

    # Step 3: Update AIMS manifest
    aims_manifest.add_content_access_right(
        source="publisher.com",
        license_url=result.active_licenses[0],
        terms=license.terms
    )

    # Step 4: Access content with telemetry
    session = telemetry.start_session(
        agent_id=aims_manifest.did,
        content_scope={"licenses": result.active_licenses}
    )

    content = fetch_content("https://publisher.com/article")
    session.log_event("content_retrieved", content_id="publisher.com/article")

    # Use content in response
    response = generate_response(content)
    session.log_event("content_cited", content_id="publisher.com/article")

    session.end()
```

### Use Case 2: Agent-to-Agent Content Sharing

```python
# Agent A checks if it can share content with Agent B
agent_b_manifest = fetch_aims_manifest(agent_b_did)

# Check Agent B's training data provenance
if agent_b_manifest.training_data.has_license("proprietary"):
    # Can't share; Agent B might train on it
    return "Access denied - licensing conflict"

# Check redistribution rights
if not my_content_access.allows_redistribution_to(agent_b_did):
    return "Access denied - no redistribution rights"

# Safe to share - log in telemetry
session.log_event("content_shared", {
    "recipient": agent_b_did,
    "content_id": content_id
})
```

## For Developers

### If you're building an AI agent:

1. **Use PolicyCheck** to check publisher policies before accessing content
2. **Publish AIMS manifest** declaring your training data and access rights
3. **Send Telemetry events** tracking content usage
4. **Verify other agents** via AIMS manifests before sharing licensed content

### If you're a content publisher:

1. **Publish robots.txt, RSL, TDMRep** to declare your policies
2. **Monitor Telemetry** to see how your content is being used
3. **Request AIMS manifests** to verify agent identities
4. **Use attribution data** to inform licensing and partnerships

### If you're building attribution infrastructure:

1. **Consume Telemetry events** from AI agents
2. **Verify AIMS manifests** for agent identity
3. **Check publisher policies** via PolicyCheck for compliance
4. **Calculate attribution** linking content to outcomes

## Roadmap Alignment

### PolicyCheck Priorities

1. ✅ Robots.txt + RSL detection (v0.1)
2. 🚧 TDMRep support (v0.2) - Critical for AI training compliance
3. 🚧 security.txt support (v0.3) - Contact discovery
4. 📋 AIMS manifest verification (v0.4) - Verify agent identity before sharing
5. 📋 Telemetry integration (v0.5) - Auto-populate content_scope metadata

## Contributing

PolicyCheck is one piece of the OpenAttribution ecosystem. Contributions welcome across:

- **PolicyCheck**: Publisher policy checking
- **AIMS**: Agent identity and manifests
- **Telemetry**: Attribution tracking

See [openattribution.org](https://openattribution.org) for the full initiative.

## References

- **OpenAttribution Initiative**: https://openattribution.org
- **AIMS Specification**: ../aims/SPECIFICATION.md
- **Telemetry Specification**: ../telemetry/SPECIFICATION.md
- **RSL Standard**: https://rslstandard.org/rsl
- **RFC 9309** (Robots.txt): https://www.rfc-editor.org/rfc/rfc9309.html
- **W3C DIDs**: https://www.w3.org/TR/did-core/
- **A2A Protocol**: https://a2a.com

---

**PolicyCheck is the compliance checker that helps AI agents respect publisher policies and integrate with AIMS + Telemetry for transparent, attributable content usage.**
