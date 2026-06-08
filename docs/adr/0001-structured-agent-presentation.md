# Structured Agent Presentation

Agent clients should receive structured presentation material from `tty-pet` MCP tools rather than relying only on plugin skill prose. This keeps Codex, Claude Code, and scripts aligned around the same pet reaction, state summary, and presentation hint while preserving the agent's role as a Pet Interpreter instead of a development coach.

**Considered Options**

- Strengthen plugin skills only.
- Return structured MCP tool payloads with reaction and presentation fields.

**Consequences**

MCP responses become a small product contract. Tests should cover both the recorded pet interaction and the presentation material returned to agents.

Structured payloads should include enough material for an agent to answer naturally without inventing state:

- `event`: the pet interaction or workflow event that was recorded.
- `reaction`: mood, short phrase, and optional motion hint.
- `state`: factual bond, last test status, and last event values.
- `presentation`: short Korean and English text that can be used directly or lightly adapted.

`tty-pet status --json` remains factual machine output. MCP tools are agent-facing: `tty_pet_status` and `tty_pet_event` should both return factual state plus reaction and presentation material.

Presentation text should start deterministic. Event-specific wording should provide personality first; randomized or rotating presentation can be added later if the deterministic responses still feel stale.

MCP tools should provide ready-to-use Korean and English presentation text plus adaptation rules. Agents may lightly adapt wording to the current conversation, but they must not invent state, give development advice, or expand beyond one or two short sentences.

Status presentation should mention the last event only when it is recent, starting with a two-minute window. Otherwise it should focus on mood and bond so old events do not feel like current pet behavior.
