import { PortfolioData } from "./types";

export function introPrompt(portfolio?: PortfolioData, userAddress?: string) {
  return `
  <knowledge>
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
    amount: z.number().nullable(),
    percentage: z.number().nullable(),
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
    value: z.number(),
  });

  const PipelineStepSchema = z.object({
    action: PipelineActionSchema,
    conditions: z.array(PipelineConditionSchema),
  });

  const PipelineSchema = z.object({
    steps: z.array(PipelineStepSchema),
  });

  now when generating a pipeline, put it into <pipeline></pipeline> tags

  always include the tags! otherwise the pipeline will not be executed
  </knowledge>
  <context>address: ${userAddress} ${JSON.stringify(
    portfolio
  )} (prices in USD)</context>
  `;
}
