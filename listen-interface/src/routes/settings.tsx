import { usePrivy } from "@privy-io/react-auth";
import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { ChatSelector } from "../components/ChatSelector";
import { ConnectedAccounts } from "../components/ConnectedAccounts";
import { WalletAddresses } from "../components/WalletAddresses";
import { useChatType } from "../hooks/useChatType";

export const Route = createFileRoute("/settings")({
  component: Settings,
});

function Settings() {
  const { user } = usePrivy();
  const { chatType, setChatType } = useChatType();
  const [quickBuyAmount, setQuickBuyAmount] = useState<number>(0.1);

  // Load saved quick buy amount from localStorage
  useEffect(() => {
    const savedAmount = localStorage.getItem("quickBuyAmount");
    if (savedAmount) {
      setQuickBuyAmount(parseFloat(savedAmount));
    }
  }, []);

  // Save quick buy amount to localStorage
  const handleAmountChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    if (!isNaN(value) && value > 0) {
      setQuickBuyAmount(value);
      localStorage.setItem("quickBuyAmount", value.toString());
    }
  };

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">Settings</h1>

      <h2 className="text-lg font-bold mb-2 mt-4">Quick Buy</h2>
      <div className="bg-black/40 backdrop-blur-sm border border-purple-500/30 rounded-lg p-4 mb-4">
        <label className="block text-sm text-purple-100 mb-2">
          Default SOL amount for quick buy:
        </label>
        <div className="flex items-center">
          <input
            type="number"
            value={quickBuyAmount}
            onChange={handleAmountChange}
            min="0.01"
            step="0.01"
            className="bg-black/60 border border-purple-500/30 rounded-lg px-3 py-2 text-purple-100 w-24 focus:outline-none focus:border-purple-500"
          />
          <span className="ml-2 text-purple-300">SOL</span>
        </div>
        <p className="text-xs text-purple-300 mt-2">
          This amount will be used when clicking the quick buy button on tokens
        </p>
      </div>

      <h2 className="text-lg font-bold mb-2 mt-4">Mode</h2>
      <ChatSelector selectedChat={chatType} onSelectChat={setChatType} />

      <h2 className="text-lg font-bold mb-2 mt-4">Wallet Addresses</h2>
      <WalletAddresses />

      <h2 className="text-lg font-bold mb-2 mt-4">Connected Accounts</h2>
      {user && <ConnectedAccounts user={user} />}
    </div>
  );
}
