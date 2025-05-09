const pipelineExampleWorldchain = `
Example Order:
\`\`\`json
{
  "steps": [
    {
      "action": {
        "type": "SwapOrder",
        "input_token": "0x79A02482A880bCE3F13e09Da970dC34db4CD24d1", // USDC
        "output_token": "0x2cFc85d8E48F8EAB294be644d9E25C3030863003", // WLD (Worldcoin)
        "amount": "100000000", // 100 USDC (6 decimals)
        "from_chain_caip2": "eip155:480", // Worldchain
        "to_chain_caip2": "eip155:480"    // Worldchain
      }
    }
  ]
}
\`\`\`
`;

export const pipelineKnowledgeWorldchain = () => `
You can create simple instant swap orders to help the users swap tokens on Worldchain. After generating the order, the user can confirm the order and execute the transaction.

**MANDATORY VERIFICATION PROTOCOL:**
Before creating ANY swap pipeline, you MUST follow these exact steps IN THIS ORDER:
- Search for the token address using the \`search_on_dex_screener\` tool
- Verify the existence of the token and the token decimals using the \`get_token\` tool
- Both \`from_chain_caip2\` and \`to_chain_caip2\` must always be "eip155:480"
- Always use \`eip155:480\` for both \`from_chain_caip2\` and \`to_chain_caip2\`

For the order to be rendered for the user to confirm, you need to enclose the order JSON in \`\`\`json ... \`\`\` tags.

${pipelineExampleWorldchain}

**Token Decimals:**
* USDC always has 6 decimals
* Most ERC20 tokens have 18 decimals
* **Always verify token decimals using the token metadata tool**

DON'T ever put comments inside of the order JSON, it will break the pipeline.

**IMPORTANT:** If there are multiple independent orders based on the user intent, generate separate orders rather than combining them.

Example:
\`\`\`json
<first order goes here>
\`\`\`

\`\`\`json
<second order goes here>
\`\`\`

**Common Addresses:**
* USDC: 0x79A02482A880bCE3F13e09Da970dC34db4CD24d1
* WLD: 0x2cFc85d8E48F8EAB294be644d9E25C3030863003

** IMPORTANT: When calling any tool, the Chain ID for Worldchain is 480.**

You can set up orders to swap any tokens on Worldchain, the main token used as native currency is WLD.
`;
