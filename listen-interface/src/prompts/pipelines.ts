import { addressBook, caip2Map } from "../hooks/util";

const pipelineExample = `
Example Pipeline:
{
  "steps": [
    {
      // Example 1: Solana Swap of SOL into USDC (executed immediately)
      "action": {
        "type": "SwapOrder",
        "input_token": "So11111111111111111111111111111111111111112", // SOL mint address
        "output_token": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC mint address (Solana)
        "amount": "1000000000" // 1 SOL (10^9)
        // from_chain_caip2/to_chain_caip2 omitted, defaults to Solana
      }
      // No "conditions", executes immediately
    },
    {
      // Example 2: EVM Swap (conditional)
      "action": {
        "type": "SwapOrder",
        "input_token": "0x0000000000000000000000000000000000000000", // Native ETH placeholder
        "output_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC address (Ethereum Mainnet)
        "amount": "500000000000000000", // 0.5 ETH (10^18)
        "from_chain_caip2": "eip155:1", // Required for EVM/cross-chain
        "to_chain_caip2": "eip155:1"   // Required for EVM/cross-chain
      },
      "conditions": [
        {
          "type": "PriceBelow",
          "asset": "0x0000000000000000000000000000000000000000", // Native ETH placeholder
          "value": 1950 // ETH price in USD
        }
      ]
    },
    {
      // Example 3: Notification (conditional)
      "action": {
        "type": "Notification",
        "input_token": "So11111111111111111111111111111111111111112", // SOL mint address
        "message": "Notify me when SOL price goes above $160"
      },
      "conditions": [
        {
          "type": "PriceAbove",
          "asset": "So11111111111111111111111111111111111111112", // SOL mint address
          "value": 160
        }
      ]
    }
  ]
}
`;

export const pipelineKnowledge = () => `
  You can create series of steps that user approves with a click to execute
  interactions which involve multiple steps, as well as simple swaps.
  Here is an example format for the pipeline:

  ${pipelineExample}

  CAIP2 map (for Solana, leave blank):
  ${JSON.stringify(caip2Map)}

  Common addresses:
  ${JSON.stringify(addressBook)}

  Key Points:
  - The pipeline is a JSON object with a "steps" array.
  - Each step has an "action" object and optional "conditions" array.
  - Action types: "SwapOrder", "Notification".
  - Condition types: "PriceAbove", "PriceBelow", "Now".
  - For "SwapOrder", specify input/output tokens (address/mint) and amount (considering decimals). Amount can be null if you want to specify "all".
    - For EVM or cross-chain swaps, you MUST include "from_chain_caip2" and "to_chain_caip2".
    - For Solana-only swaps, omit "from_chain_caip2" and "to_chain_caip2"; it will default to Solana.
  - For "Notification", specify the token (input_token) and a message.
  - For conditions, specify type, asset (token address/mint), and value (price in USD). "Now" type doesn't use "value".
  - If a step should execute immediately (or immediately after the previous step completes), omit the "conditions" key entirely.

  When generating a pipeline, put the JSON object into <pipeline></pipeline> tags.
  Always include the <pipeline></pipeline> tags! Otherwise the pipeline will not be rendered.
`;
