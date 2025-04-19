const chainIdToGeckoTerminalId = {
  "1": "eth",
  "8453": "base",
  // TODO add remaining
};

export function GeckoTerminalChart({
  pairAddress,
  chainId,
  timeframe,
}: {
  pairAddress: string;
  chainId: string;
  timeframe: string;
}) {
  if (
    !chainIdToGeckoTerminalId[chainId as keyof typeof chainIdToGeckoTerminalId]
  ) {
    return null;
  }
  const geckoTerminalId =
    chainIdToGeckoTerminalId[chainId as keyof typeof chainIdToGeckoTerminalId];
  return (
    <iframe
      height="100%"
      width="100%"
      title="GeckoTerminal Embed"
      src={`https://www.geckoterminal.com/${geckoTerminalId}/pools/${pairAddress}?embed=1&info=0&swaps=0&grayscale=0&light_chart=0&chart_type=price&resolution=${timeframe}`}
      allow="clipboard-write"
    />
  );
}
