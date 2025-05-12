import { useState } from "react";
import { EvmWalletCreation } from "./EvmWalletCreation";
import { SolanaWalletCreation } from "./SolanaWalletCreation";

export function DelegateActionButton() {
  const [solanaError, _setSolanaError] = useState<string | null>(null);
  const [evmError, _setEvmError] = useState<string | null>(null);

  return (
    <div className="flex flex-col gap-2">
      <SolanaWalletCreation error={solanaError} />
      <EvmWalletCreation error={evmError} />
    </div>
  );
}
