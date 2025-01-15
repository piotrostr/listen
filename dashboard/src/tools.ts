import Anthropic from "@anthropic-ai/sdk";

export const tools: Anthropic.Tool[] = [
  {
    name: "swap_tokens",
    description:
      "Perform a token swap on Solana between two SPL tokens. The amount is the input amount in raw token units. Slippage is in basis points (1 bp = 0.01%).",
    input_schema: {
      type: "object",
      properties: {
        input_mint: {
          type: "string",
          description: "The input token mint address",
        },
        output_mint: {
          type: "string",
          description: "The output token mint address",
        },
        amount: {
          type: "integer",
          description: "The input amount in raw token units",
          minimum: 0,
        },
        slippage: {
          type: "integer",
          description:
            "Maximum allowed slippage in basis points (1 bp = 0.01%), max .5% is OK",
          minimum: 0,
        },
      },
      required: ["input_mint", "output_mint", "amount", "slippage"],
    },
  },
];
