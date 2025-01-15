import { tools } from "../tools";
import { useListen } from "./useListen";
import { SwapParams } from "./schema";

export type ToolInputs = SwapParams;

export const useTools = () => {
  const { swap } = useListen();
  async function handleToolUse(tool: string, toolInputs: ToolInputs) {
    switch (tool) {
      case "swap_tokens":
        return await swap({
          amount: toolInputs.amount,
          slippage: toolInputs.slippage,
          input_mint: toolInputs.input_mint,
          output_mint: toolInputs.output_mint,
        });
    }
  }
  return { tools, handleToolUse };
};
