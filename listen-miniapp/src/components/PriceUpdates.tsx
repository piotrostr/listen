import { useEffect, useMemo, useState } from "react";
import { useTokenStore } from "../store/tokenStore";
import { TokenTile } from "./TokenTile";

export function PriceUpdates() {
  const {
    tokenMap,
    filterAndSortTokens,
    isListFrozen,
    setIsListFrozen,
    marketCapFilter,
    volumeFilter,
  } = useTokenStore();
  const [frozenTokens, setFrozenTokens] = useState<any[]>([]);

  // Convert to array once and memoize that result separately
  const tokenArray = useMemo(() => Array.from(tokenMap.values()), [tokenMap]);

  // Simply use the store's filtering function directly
  const currentTokens = useMemo(() => {
    return filterAndSortTokens(tokenArray, marketCapFilter, volumeFilter, 20);
  }, [tokenArray, marketCapFilter, volumeFilter, filterAndSortTokens]);

  // Keep frozen tokens updated with current tokens when not frozen
  useEffect(() => {
    if (!isListFrozen) {
      setFrozenTokens(currentTokens);
    }
  }, [currentTokens, isListFrozen]);

  // Use frozen tokens when list is frozen, otherwise use current tokens
  const topTokens = isListFrozen ? frozenTokens : currentTokens;

  const handleMouseEnter = () => {
    setIsListFrozen(true);
  };

  const handleMouseLeave = () => {
    setIsListFrozen(false);
  };

  return (
    <div className="h-full font-mono overflow-y-auto scrollable-container">
      <div onMouseEnter={handleMouseEnter} onMouseLeave={handleMouseLeave}>
        {topTokens.map((token) => (
          <TokenTile key={token.pubkey} token={token} />
        ))}
      </div>
    </div>
  );
}
