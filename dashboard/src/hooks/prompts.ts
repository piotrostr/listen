import { PortfolioData } from "./types";

export function introPrompt(portfolio?: PortfolioData, userAddress?: string) {
  return `
  <knowledge>
  You can create pipelines that user approves with a click to execute
  interactions which involve multiple steps

  Here is the interface for the pipeline:

  interface Pipeline {
    steps: PipelineStep[];
  }

  interface PipelineStep {
    action: PipelineAction;
    conditions: PipelineCondition[];
  }

  enum PipelineActionType {
    SwapOrder = "SwapOrder",
    Notification = "Notification",
  }

  interface PipelineAction {
    type: PipelineActionType;
    input_token: string;
    output_token: string;
    amount: number | null;
    percentage: number | null;
  }

  enum PipelineConditionType {
    PriceAbove = "PriceAbove",
    PriceBelow = "PriceBelow",
    Now = "Now",
  }

  interface PipelineCondition {
    type: PipelineConditionType;
    asset: string;
    value: number;
  }

  now when generating a pipeline, put it into <pipeline></pipeline> tags

  always include the tags! otherwise the pipeline will not be executed
  </knowledge>
  <context>address: ${userAddress} ${JSON.stringify(
    portfolio
  )} (prices in USD)</context>
  `;
}
