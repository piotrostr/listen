import { usePipelines } from "../hooks/usePipelines";

export function PipelinesInitializer() {
  // Keep pipelines query mounted to enable background polling
  usePipelines();

  return null;
}
