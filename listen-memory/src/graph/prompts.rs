pub const EXTRACT_ENTITIES_PROMPT: &str = "
You are a smart assistant specialized in extracting ONLY high-confidence, verifiable connections from crypto-related text. Focus on:

1. Social Identity Links:
   - Twitter handle -> address
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
4. For claims, require specific evidence/posts

Example Valid:
{{
    \"source\": \"@arcdotfun\",
    \"relationship\": \"has_address\",
    \"target\": \"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump\",
}}

{{
    \"source\": \"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump\",
    \"relationship\": \"trades_at_lp\",
    \"target\": \"J3b6dvheS2Y1cbMtVz5TCWXNegSjJDbUKxdUVDPoqmS7\",
    \"context\": \"ARC/WSOL pair traded on Raydium\"
}}

{{
    \"source\": \"@blknoiz06\",
    \"relationship\": \"is_affiliated_with\",
    \"target\": \"@BullpenFi\",
    \"context\": \"blknoiz06 is a member of BullpenFi\"
}}

{{
    \"source\": \"BullpenFi/1909336899266687434\",
    \"relationship\": \"mentions\",
    \"target\": \"@_Fullport\",
    \"context\": \"@salxyz walks viewers through Bullpen's Hyperliquid Beta\"
}}

Example Invalid:
{{
    \"source\": \"arc\",
    \"relationship\": \"related_to\",
    \"target\": \"ai_agents\",
}}

{{
    \"source\": \"ansem\",
    \"relationship\": \"knows\",
    \"target\": \"kanye\",
}}

If timestamp is available as part of the data, include it for factualness, but it's not required.
";

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
