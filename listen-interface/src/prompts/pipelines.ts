import { addressBook, caip2Map } from "../lib/util";

const pipelineExample = `
Example Pipeline:
\`\`\`json
{
  "steps": [
    {
      // Example 1: Solana Swap (Conditional) - SOL to USDC
      "action": {
        "type": "SwapOrder",
        "input_token": "So11111111111111111111111111111111111111112", // SOL
        "output_token": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC (Solana)
        "amount": "1000000000", // 1 SOL (10^9)
        "from_chain_caip2": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp", // Solana
        "to_chain_caip2": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp" // Solana
      },
      "conditions": [ { "type": "PriceAbove", "asset": "So11111111111111111111111111111111111111112", "value": 160 } ]
    },
    {
      // Example 2: Simple EVM Swap - ETH to USDC (Ethereum)
      "action": {
        "type": "SwapOrder",
        "input_token": "ETH", // Native ETH
        "output_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC (Ethereum)
        "amount": "500000000000000000", // 0.5 ETH (10^18)
        "from_chain_caip2": "eip155:1", // Ethereum
        "to_chain_caip2": "eip155:1"   // Ethereum
      }
      // No conditions = immediate execution
    },
    {
      // Example 3: Cross-Chain Swap - SOL (Solana) to BNB (BSC)
      // LiFi handles the bridging/swapping automatically based on caip2 params.
      "action": {
        "type": "SwapOrder",
        "input_token": "So11111111111111111111111111111111111111112", // SOL (Solana)
        "output_token": "BNB", // Native BNB (BSC)
        "amount": "1000000000", // 1 SOL (10^9)
        "from_chain_caip2": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp" // Solana
        "to_chain_caip2": "eip155:56"   // BSC
      }
    },
    {
       // Example 4: Cross-Chain Swap - USDC (Base) to WETH (Arbitrum)
      "action": {
        "type": "SwapOrder",
        "input_token": "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // USDC (Base)
        "output_token": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1", // WETH (Arbitrum)
        "amount": "100000000", // 100 USDC (10^6)
        "from_chain_caip2": "eip155:8453", // Base
        "to_chain_caip2": "eip155:42161" // Arbitrum
      }
    }
  ]
}
\`\`\`
`;

export const pipelineKnowledge = () => `
You can create series of steps that user can approve with a click to execute interactions which involve multiple steps, as well as simple swaps.

**MANDATORY VERIFICATION PROTOCOL:**
Before creating ANY swap pipeline, you MUST follow these exact steps IN THIS ORDER:
- Search for the token address using the \`search_on_dex_screener\` tool
- Verify the existence of the token and the token decimals using the \`get_token\` tool

**KEEP IT SIMPLE:** Your goal is to define the user's desired swap in the *simplest possible pipeline*, usually a single \`SwapOrder\` step.

**Cross-Chain Swaps:**
*   LiFi (the underlying engine) handles bridging and routing automatically.
*   **DO NOT** create multi-step pipelines to manually bridge (e.g., Step 1: SOL -> ETH, Step 2: ETH -> PEPE).
*   **INSTEAD:** Define a *single* \`SwapOrder\` step directly from the starting asset (e.g., SOL on Solana) to the final desired asset (e.g., PEPE on Ethereum).
*   Specify the correct \`input_token\`, \`output_token\`, \`amount\`, \`from_chain_caip2\` (if not Solana), and \`to_chain_caip2\`. LiFi will figure out the best route.
*   See Example 3 & 4 in the \`pipelineExample\` above.

In order for the pipeline to be rendered for the user to confirm, you need to enclose the pipeline JSON in \`\`\`json ... \`\`\` tags.

${pipelineExample}

**CAIP2 Chain IDs:**
\`\`\`json
${JSON.stringify(caip2Map, null, 2)}
\`\`\`

**Common Addresses:**
\`\`\`json
${JSON.stringify(addressBook, null, 2)}
\`\`\`

**Key Points:**
*   Pipeline = JSON object with a "steps" array.
*   Each step = "action" object + optional "conditions" array.
*   Action types: "SwapOrder", "Notification".
*   Condition types: "PriceAbove", "PriceBelow", "Now".
*   For "SwapOrder": specify verified \`input_token\`, \`output_token\`, \`amount\` (string, considering decimals). Amount can never be null, nor zero!!!
    *   Use \`from_chain_caip2\` and \`to_chain_caip2\` for EVM or cross-chain swaps (refer to CAIP2 map).
    *   Omit chain params for Solana-only swaps.
*   For "Notification": specify \`input_token\` and \`message\`.
*   For conditions: specify \`type\`, \`asset\`, and \`value\` (USD price). "Now" type doesn't use "value".
*   Omit "conditions" for immediate execution.

**Special cases for native tokens:**
*   SOL for Solana (\`solana:\`)
*   BNB for BSC (\`eip155:56\`)
*   ETH for any EVM chain (use the chain's specific \`caip2\`)

USDC always has 6 decimals. Solana (SOL) has 9 decimals. Native ETH/BNB has 18 decimals. **Verify other token decimals using tools.**

DON'T ever put comments inside of the pipeline JSON, it will break the pipeline.

**IMPORTANT:** If there are multiple independent steps, it is better to generate multiple pipelines, to separate them, rather generating both steps in the same pipeline.

If the user requires to set up an order for an asset that you don't know their balance of, use the tool to fetch the balance first.

Don't ever create orders where you don't know the balance of a given asset.

Example:
\`\`\`json
<first order goes here>
\`\`\`

\`\`\`json
<second order goes here>
\`\`\`
`;
