import { addressBook, caip2Map } from "./util";

const pipelineKnowledgeEvm = `
  You can create pipelines that user approves with a click to execute
  interactions which involve multiple steps

  Here is the format for the pipeline defined as zod validators:

  For Solana, the caip2 is "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"

  enum PipelineActionType {
    SwapOrder = "SwapOrder",
    Notification = "Notification",
  }

  enum PipelineConditionType {
    PriceAbove = "PriceAbove",
    PriceBelow = "PriceBelow",
    Now = "Now",
  }

  const SwapOrderActionSchema = z.object({
    type: z.literal(PipelineActionType.SwapOrder),
    input_token: z.string(), // address or mint
    output_token: z.string(), // address or mint
    // accounting for decimals, e.g. 1 sol = 10^9 lamports, 1 eth = 10^18 wei
    amount: z.string().nullable(), 
    from_chain_caip2: z.string(), 
    to_chain_caip2: z.string(),
  });

  const NotificationActionSchema = z.object({
    type: z.literal(PipelineActionType.Notification),
    input_token: z.string(), // address or mint
    message: z.string(),
  });

  const PipelineActionSchema = z.discriminatedUnion("type", [
    SwapOrderActionSchema,
    NotificationActionSchema,
  ]);

  const PipelineConditionSchema = z.object({
    type: z.nativeEnum(PipelineConditionType),
    asset: z.string(), // address or mint
    value: z.number(), // Now can take any value, its not used
  });

  const PipelineStepSchema = z.object({
    action: PipelineActionSchema,
    conditions: z.array(PipelineConditionSchema).optional(), // optional, if not provided, executes immediately
  });

  const PipelineSchema = z.object({
    steps: z.array(PipelineStepSchema),
  });

  now when generating a pipeline, put it into <pipeline></pipeline> tags

  always include the tags! otherwise the pipeline will neither be rendered for the user to see nor executed
`;

const pipelineKnowledgeSolana = `
  You can create pipelines that user approves with a click to execute
  interactions which involve multiple steps

  Here is the format for the pipeline defined as zod validators:

  enum PipelineActionType {
    SwapOrder = "SwapOrder",
    Notification = "Notification",
  }

  enum PipelineConditionType {
    PriceAbove = "PriceAbove",
    PriceBelow = "PriceBelow",
    Now = "Now",
  }

  const SwapOrderActionSchema = z.object({
    type: z.literal(PipelineActionType.SwapOrder),
    input_token: z.string(), // address or mint
    output_token: z.string(), // address or mint
    // accounting for decimals, e.g. 1 sol = 10^9, 1 usdc = 10^6
    amount: z.string().nullable(), 
  });

  const NotificationActionSchema = z.object({
    type: z.literal(PipelineActionType.Notification),
    input_token: z.string(), // address or mint
    message: z.string(),
  });

  const PipelineActionSchema = z.discriminatedUnion("type", [
    SwapOrderActionSchema,
    NotificationActionSchema,
  ]);

  const PipelineConditionSchema = z.object({
    type: z.nativeEnum(PipelineConditionType),
    asset: z.string(), // address or mint
    value: z.number(), // Now can take any value, its not used
  });

  const PipelineStepSchema = z.object({
    action: PipelineActionSchema,
    conditions: z.array(PipelineConditionSchema).optional(), // optional, if not provided, executes immediately
  });

  const PipelineSchema = z.object({
    steps: z.array(PipelineStepSchema),
  });

  now when generating a pipeline, put it into <pipeline></pipeline> tags

  always include the tags! otherwise the pipeline will neither be rendered for the user to see nor executed
`;

export function systemPromptEvm(
  portfolio: {
    chain: string;
    address: string;
    amount: string;
    name: string;
    symbol: string;
    decimals: number;
  }[],
  walletAddress: string,
  pubkey: string
) {
  return `
  ${pipelineKnowledgeEvm}
  <guidelines>
  Often, there will be tokens on other chains that mimick the "original" token, even in the user portfolio.
  The original token is the one with highest liquidity and volume.
  Always assume the user wants the OG token, unless specified otherwise.
  </guidelines>
  <context>Address EVM: ${walletAddress}; Address SOL: ${pubkey}; ${JSON.stringify(
    portfolio
  )} (prices in USD)</context>
  <chain_caip2_map>
  ${JSON.stringify(caip2Map)}
  </chain_caip2_map>
  <address_book>
  ${JSON.stringify(addressBook)}
  </address_book>
  `;
}

export function systemPromptSolana(
  solanaPortfolio: {
    chain: string;
    address: string;
    amount: string;
    name: string;
    symbol: string;
    decimals: number;
  }[],
  pubkey: string
) {
  return `
  <context>Address SOL: ${pubkey}; Portfolio: ${JSON.stringify(solanaPortfolio)} (prices in USD)</context>
  <address_book>
  ${JSON.stringify(addressBook["solana"])}
  </address_book>
  <knowledge>
  ${pipelineKnowledgeSolana}
  <errors>
    0x1771: program error when slippage tolerance is too low, this can be fixed by increasing the slippage tolerance or a retry
  </errors>
  </knowledge>
  <guidelines>
  Be friendly, concise, and helpful when discussing the user's Solana portfolio.
  Use conversational language and avoid overly technical jargon unless the user demonstrates advanced knowledge.
  Frame suggestions as helpful options rather than pushing the user toward any specific action.
  </guidelines>
  <limitations>
  Only discuss limitations if the user would ask about something you cannot do
  - adding liquidity is currently not supported, jupiter liquidity proivder is an option you could suggest instead
  </limitations>
  `;
}
