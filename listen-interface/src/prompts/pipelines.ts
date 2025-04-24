import { addressBook, caip2Map } from "../hooks/util";

const pipelineExample = `
Example Pipeline:
{
  "steps": [
    {
      // Example 1: Solana Swap of SOL into USDC (executed if SOL price is above $160)
      "action": {
        "type": "SwapOrder",
        "input_token": "So11111111111111111111111111111111111111112", // SOL mint address
        "output_token": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC mint address (Solana)
        "amount": "1000000000" // 1 SOL (10^9)
        // from_chain_caip2/to_chain_caip2 omitted, defaults to Solana
      },
      "conditions": [
        {
          "type": "PriceAbove",
          "asset": "So11111111111111111111111111111111111111112", // SOL mint address
          "value": 160 // SOL price in USD
        }
      ]
    },
    {
      // Example 2: EVM Swap (conditional)
      "action": {
        "type": "SwapOrder",
        "input_token": "ETH", // Native ETH placeholder
        "output_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC address (Ethereum Mainnet)
        "amount": "500000000000000000", // 0.5 ETH (10^18)
        "from_chain_caip2": "eip155:1", // Required for EVM/cross-chain
        "to_chain_caip2": "eip155:1"   // Required for EVM/cross-chain
      },
      // No "conditions", executes immediately
      // NOTE: "conditions" is only available for Solana tokens, there is currently no
      // live price feed for EVM tokens in the Listen order engine
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
    },
    {
      // Example 4: Bridge
      "action": {
        "type": "SwapOrder",
        "input_token": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC mint address (Solana)
        "output_token": "BNB", // BNB address (BSC Mainnet)
        "amount": "30000000", // 30 USDC (10^6)
        "to_chain_caip2": "eip155:56"   // Required for EVM/cross-chain
      }
    }
  ]
}
`;

export const pipelineKnowledge = () => `
  You can create series of steps that user can approve with a click to execute interactions which involve multiple steps, as well as simple swaps.

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
  - IMPORTANT: Solana native tokens sometimes fail with bridges to other chains, swap to USDC on Solana first, then swap into the desired token. If this is the case, flow is swap SOL to USDC -> check balance -> swap to desired token. DO IT IN TWO SEPARATE PIPELINES! NOT JUST STEPS!
  - VERY IMPORTANT: as per above, if you don't do this in this sequence, you won't know how much USDC you can allocate. Solana swaps are instant, perform the above sequence and only then continue to move funds to other chains.

  Special cases for native tokens is ETH for any EVM chain instead of address and BNB for BSC, anything else has to be an address.

  In order for the pipeline to be rendered for the user to confirm, you need to enclose the pipeline \`\`\`json\`\`\` tags.
`;
