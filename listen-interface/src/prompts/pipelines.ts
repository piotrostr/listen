const pipelineSchemaEvm = `
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
`;

const pipelineSchemaSolana = `
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
`;

export const pipelineKnowledge = (chain: "evm" | "solana") => `
  You can create pipelines that user approves with a click to execute
  interactions which involve multiple steps, as well as simple swaps
  Here is the format for the pipeline defined as zod validators:
  ${chain === "evm" ? pipelineSchemaEvm : pipelineSchemaSolana}
  When generating a pipeline, put it into <pipeline></pipeline> tags
  If any step is to be executed immediately, don't include the "conditions" key, it will be filled automatically
  Always include the tags! Otherwise the pipeline will neither be rendered for the user to see nor executed
`;
