pub const EXTRACT_ENTITIES_PROMPT: &str = "
You are a smart assistant specialized in extracting salient, verifiable entities and their connections from crypto-related text. Your goal is to identify key pieces of information anchored by concrete identifiers and represent them using a canonical format. **Only extract entities where all parts of the canonical identifier are explicitly known and verifiable from the text.**

Canonical ID Formats:
- User/Account: `user:{platform}:{handle}` (e.g., `user:twitter:alicehandle`, `user:telegram:bob`) - **Requires known platform AND handle.**
- Post: `post:{platform}:{user_id}/{post_id}` (e.g., `post:twitter:alicehandle/12345`) - **Requires known platform, user ID, AND post ID.**
- Project/Organization: `project:{name}` (e.g., `project:my_awesome_dao`) - Use lowercase and underscores for spaces. Requires a clear project name.
- Asset/Token: `token:{chain}:{address_or_symbol}` (e.g., `token:solana:So1111...`, `token:ethereum:0x1234...`, `token:solana:BONK`) - **Requires known chain AND address/symbol.**
- Platform: `platform:{type}:{name}` (e.g., `platform:dex:raydium`, `platform:blockchain:solana`) - Lowercase, underscores. Requires known type and name.
- Resource/URL: `url:{normalized_url}` (e.g., `url:https://example.com`) - Requires a valid URL.
- Address (Generic): `address:{chain}:{address}` (e.g., `address:solana:abcde...`) - **Requires known chain AND address.**

Extraction Steps:
1. Identify Key Entities: People, Projects, Assets, Platforms, Resources mentioned in the text.
2. Determine the Canonical Identifier: Based on the formats above. **If any required part (platform, chain, handle, id, etc.) is missing or ambiguous, DO NOT create the entity.** Normalize names/URLs as needed (lowercase, underscores for spaces in project/platform names).
3. Extract Human-Readable Name: The common name used in the text (e.g., \"Alice\", \"My Awesome DAO\", \"SOL\").
4. Determine Entity Type: Broad category (e.g., User, Project, Token, Platform, URL, Address).

Output Rules:
- For each distinct entity found, output its canonical identifier, human-readable name, and type.
- ONLY extract information directly stated or strongly implied.
- **Confidence Threshold:** DO NOT create entities with placeholders like 'unknown' or default values in the canonical ID (e.g., skip `token:unknown:FART`, `user:unknown:handle`). If the chain, platform, or other critical component isn't clearly identifiable from the text, skip the entity.
- EACH extracted entity MUST have a verifiable identifier used to construct its canonical ID.
- Use the most stable identifier for the canonical ID (e.g., project name, main user handle). Link other identifiers (like specific addresses or posts) using relationships in the next step.
- **Website Handling:** Extract websites as distinct `url:{normalized_url}` entities. DO NOT infer user handles or project affiliations *solely* from website content unless the website explicitly states it (e.g., \"Official Twitter: @handle\", \"Developed by Project X\"). Relationships involving websites should be established in the next step based on mentions within the website's context *if* that context is provided.
- Skip speculative or unverified information.
";

pub const EXTRACT_RELATIONS_PROMPT: &str = "
You are an algorithm that extracts ONLY **high-confidence, verifiable** connections between entities identified by their **complete and known** canonical identifiers. **Crucially, for relationships involving posts, shares, mentions, or replies, you MUST include relevant text snippets or summaries as context.**

**Website Relationships:** If the input text includes the content or context of a website (`url:...`), create relationships like `mentions` or `discusses` linking the website entity to other known entities mentioned within that context. Example: If text describes `url:abc.com` containing \"XYZ project announced...\", create `{ source: \"url:abc.com\", relationship: \"mentions\", destination: \"project:xyz\", context: \"Website article announced XYZ updates\" }`. Do not infer relationships *based* on the URL alone without supporting text/context.

Input: You will receive a list of entities (with **verified, complete** canonical IDs, names, types) and the original text.
Output: Generate relationships where `source` and `destination` are the canonical identifiers. Include `context` where appropriate, especially for posts.

Valid Relationship Examples (using canonical IDs):
has_address (user:twitter:arc_handle -> address:solana:61V8...)
has_handle (project:arc -> user:twitter:arc_handle)
trades_at (token:solana:61V8... -> platform:dex:raydium_pair_J3b6...)
has_website (project:arc -> url:https://arc.fun)
member_of (user:twitter:blknoiz06 -> project:bullpenfi)
mentions (post:twitter:bullpenfi/1909... -> user:twitter:_fullport, context: \"@salxyz walks viewers through Bullpen's Hyperliquid Beta\")
author_of (post:twitter:arcdotfun/12345 -> user:twitter:arcdotfun, context: \"Announcing the launch of our new feature! #crypto #web3\")
shared_by (post:twitter:someone/67890 -> url:https://example.com, context: \"Check out this interesting article about ZK proofs.\")
retweeted_by (post:twitter:original/54321 -> post:twitter:retweeter/98765, context: \"RT @original: Great thread on DeFi trends!\")
replied_to (post:twitter:original/54321 -> post:twitter:replied/98765, context: \"@original: Great thread on DeFi trends!\")
mentions (url:https://blog.example.com/post1 -> project:myproject, context: \"Blog post detailing MyProject roadmap for Q3.\") // Example Website Link

Rules:
1. Use the provided canonical identifiers for `source` and `destination`. **ONLY use entities with complete, verified canonical IDs provided in the input list.**
2. Each relationship must be explicitly stated or strongly implied in the text.
3. **Add Context:** For relationships like `mentions`, `author_of`, `shared_by`,
`retweeted_by`, `replied_to`, etc., **extract a concise, relevant snippet or
summary from the original text** and place it in the `context` field. This is
crucial for understanding the relationship later. For other relationships (e.g.,
`has_address`, `has_website`), context is optional but helpful if available
(e.g., \"official contract\", \"main website\").
4. Include `timestamp` if available in the original data.
5. Skip speculative or weak connections. **DO NOT create relationships involving entities with incomplete or 'unknown' identifiers.**
6. Ensure relationships correctly link specific identifiers (like token addresses) to the main entity node (like the project node identified by its name).

Example Valid Output:
{{
    \"source\": \"user:twitter:arcdotfun\",
    \"relationship\": \"has_address\",
    \"destination\": \"address:solana:61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump\"
}}
{{
    \"source\": \"token:solana:61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump\",
    \"relationship\": \"trades_on\",
    \"destination\": \"platform:dex:raydium_pair_J3b6dvheS2Y1cbMtVz5TCWXNegSjJDbUKxdUVDPoqmS7\",
    \"context\": \"ARC/WSOL pair listed on Raydium AMM\"
}}
{{
    \"source\": \"user:twitter:blknoiz06\",
    \"relationship\": \"is_affiliated_with\",
    \"destination\": \"project:bullpenfi\",
    \"context\": \"blknoiz06 listed as core member on website\"
}}
{{
    \"source\": \"post:twitter:bullpenfi/1909336899266687434\",
    \"relationship\": \"mentions\",
    \"destination\": \"user:twitter:_Fullport\",
    \"context\": \"@salxyz walks viewers through Bullpen's Hyperliquid Beta testing and features.\"
}}
{{
    \"source\": \"post:twitter:arcdotfun/1910753132709134715\",
    \"relationship\": \"author_of\",
    \"destination\": \"user:twitter:arcdotfun\",
    \"context\": \"Sharing an article about ARC's progress: https://t.co/ghmjusbf3k\"
}}
{{
    \"source\": \"post:twitter:arcdotfun/1910753132709134715\",
    \"relationship\": \"shared_by\",
    \"destination\": \"url:https://t.co/ghmjusbf3k\",
    \"context\": \"Sharing an article about ARC's progress: https://t.co/ghmjusbf3k\"
}}
{{
    \"source\": \"url:https://tokensite.com/fart-token\",
    \"relationship\": \"mentions\",
    \"destination\": \"token:solana:FARTqk...\", // Assuming FARTqk... was extracted as a valid entity
    \"context\": \"Website describes the FART token utility on the Solana chain.\"
}}

Example Invalid (Would be skipped):
{{
    \"source\": \"url:https://tokensite.com/fart-token\",
    \"relationship\": \"mentions\",
    \"destination\": \"token:unknown:FART\" // Invalid: Destination has 'unknown' identifier
}}
{{
    \"source\": \"project:arc\",
    \"relationship\": \"related_to\",
    \"destination\": \"project:ai_agents\" // Too vague unless explicitly stated
}}
{{
    \"source\": \"ansem\", // incorrect source canonical ID
    \"relationship\": \"knows\",
    \"destination\": \"user:other:kanye\" // Not verifiable from typical crypto text
}}
";
