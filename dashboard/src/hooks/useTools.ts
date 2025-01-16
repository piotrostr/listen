import { tools } from "../tools";
import { useListen } from "./useListen";
import { SwapParams, SwapResponse } from "./schema";

export type ToolInputs = SwapParams;

export type ToolOutput = {
  id: string;
  type: string;
  data?: SwapResponse;
  status: "loading" | "success" | "error";
};

export const useTools = () => {
  const { swap } = useListen();
  async function handleToolUse(
    tool: string,
    toolInputs: ToolInputs,
  ): Promise<ToolOutput> {
    let res;
    switch (tool) {
      case "swap_tokens":
        res = await swap({
          amount: toolInputs.amount,
          slippage: toolInputs.slippage,
          input_mint: toolInputs.input_mint,
          output_mint: toolInputs.output_mint,
        });
    }
    if (!res) {
      return {
        id: crypto.randomUUID(),
        type: tool,
        data: res,
        status: "error",
      };
    }
    return {
      id: crypto.randomUUID(),
      type: tool,
      data: res,
      status: "success",
    };
  }
  return { tools, handleToolUse };
};
