pub const EXTRACT_ENTITIES_PROMPT: &str = "
You are a smart assistant specialized in extracting ONLY high-confidence, verifiable connections from crypto-related text. Focus on:

1. Social Identity Links:
   - Twitter handle -> wallet address
   - Twitter handle -> project/token
   - Telegram channel -> project/token

2. Token Relationships:
   - Token address -> DEX pair address
   - Token address -> official social handles
   - Token address -> official website

3. Project Connections:
   - Project name -> official contracts
   - Project name -> official social media
   - Project name -> key team members

Key Rules:
- ONLY extract when there's explicit connection (e.g. @arcdotfun -> 61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump)
- Skip speculative or unverified connections
- Each connection must have at least one concrete identifier (address/handle/URL)";

pub const EXTRACT_RELATIONS_PROMPT: &str = "
You are an algorithm that extracts ONLY high-confidence connections from crypto data. Focus on:

Valid Relationship Examples:
has_address (social -> contract)
has_handle (project -> social)
trades_at (token -> dex_pair)
has_website (project -> domain)

Rules:
1. Each relationship must have at least one verifiable identifier
2. Skip any speculative or weak connections
3. For social media, require explicit account ownership
4. For accusations/claims, require specific evidence/posts

Example Valid:
@arcdotfun has_address 61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump
arc trades_at_lp J3b6dvheS2Y1cbMtVz5TCWXNegSjJDbUKxdUVDPoqmS7
@blknoiz06 is_affiliated_with @BullpenFi

Example Invalid:
arc related_to ai_agents
ansem knows kanye";

// TODO!!! timestamp + context (minimal,
// something like this:
// {
//   "source": "BullpenFi/1909336899266687434",
//   "relationship": "mentions",
//   "target": "@_Fullport",
//   "timestamp": "2024-01-01 12:00:00",
//   "context": "@salxyz walks viewers through Bullpen's Hyperliquid Beta"
// }
//
// but it doesn't need context if its explicit, like the attribute of a project
// context is useful if multiple entities are linked

pub const DELETE_RELATIONS_PROMPT: &str = "
You are a graph memory manager focused on maintaining accurate, verifiable relationships. Your task is to identify which relationships should be deleted when new information arrives.

DELETE ONLY when:
1. Direct Contradiction:
   - An identifier points to a different official resource
   - A relationship is explicitly invalidated
   - A connection is proven obsolete or incorrect

2. Proven False:
   - Official announcement contradicts existing relationship
   - Technical evidence shows connection is invalid
   - Verifiable proof of incorrect association

DO NOT DELETE when:
1. Multiple Valid Relationships:
   - Multiple active connections exist simultaneously
   - Parallel valid identifiers are present
   - Different aspects of same relationship

2. Historical Records:
   - Past connections that remain valid
   - Time-stamped relationships with context
   - Sequential evolution of connections

Example Valid Deletion:
Old: identifier_A has_connection resource_X
New: identifier_A officially changed to resource_Y (with proof)

Example Invalid Deletion:
Old: identifier_A connects_to resource_X
New: identifier_A also connects_to resource_Y
(Both connections are valid)

Remember: Only delete when new information CONTRADICTS (not just adds to) existing relationships.";
