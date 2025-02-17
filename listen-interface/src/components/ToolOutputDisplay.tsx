import { Spinner } from "./Spinner";
import { useWaitForTransaction } from "../hooks/useWaitForTransaction";
import { ToolOutput } from "../hooks/useChat";

export function ToolOutputDisplay({
  toolOutput,
}: {
  toolOutput: ToolOutput | null;
}) {
  console.log(toolOutput);
  const signature = toolOutput?.name === "trade" ? toolOutput.result : null;

  const { data: transactionStatus, isLoading: isConfirming } =
    useWaitForTransaction(signature);

  if (!toolOutput) return null;

  const tryRenderJSON = (jsonstr: string) => {
    try {
      return JSON.stringify(JSON.parse(jsonstr), null, 2);
    } catch {
      return jsonstr;
    }
  };

  switch (toolOutput.name) {
    case "trade":
      return (
        <div className="flex items-center gap-2 text-sm">
          {isConfirming ? (
            <div className="flex flex-col gap-2">
              <div className="flex flex-row justify-start items-center gap-2">
                <Spinner />
                <span>Confirming transaction...</span>
              </div>
              <div className="text-xs break-all">
                <span className="opacity-70">View on Solscan: </span>
                <a
                  href={`https://solscan.io/tx/${signature}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-300 hover:text-blue-200 underline"
                >
                  {signature}
                </a>
              </div>
            </div>
          ) : transactionStatus?.success ? (
            <div className="flex flex-col gap-1">
              <div className="font-semibold text-green-300">
                Transaction confirmed
              </div>
              <div className="text-xs break-all">
                <span className="opacity-70">View on Solscan: </span>
                <a
                  href={`https://solscan.io/tx/${transactionStatus.signature}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-300 hover:text-blue-200 underline"
                >
                  {transactionStatus.signature}
                </a>
              </div>
            </div>
          ) : transactionStatus?.error ? (
            <div className="flex flex-col gap-1 text-red-300">
              <div className="font-semibold">Transaction failed</div>
              <span className="text-xs">Error: {transactionStatus.error}</span>
              {signature && (
                <div className="text-xs break-all">
                  <span className="opacity-70">View on Solscan: </span>
                  <a
                    href={`https://solscan.io/tx/${signature}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-300 hover:text-blue-200 underline"
                  >
                    {signature}
                  </a>
                </div>
              )}
            </div>
          ) : null}
        </div>
      );

    default:
      return (
        <pre className="whitespace-pre-wrap text-sm overflow-x-auto bg-blue-950/30 p-4 rounded-md">
          {tryRenderJSON(toolOutput.result)}
        </pre>
      );
  }
}
