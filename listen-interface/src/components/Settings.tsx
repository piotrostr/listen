import { usePrivy } from "@privy-io/react-auth";
import { useTranslation } from "react-i18next";
import { useSettings } from "../contexts/SettingsContext";
import { useChatType } from "../hooks/useChatType";
import { ChatSelector } from "./ChatSelector";
import { ConnectedAccounts } from "./ConnectedAccounts";
import { WalletAddresses } from "./WalletAddresses";

export function Settings() {
  const { user } = usePrivy();
  const { chatType, setChatType } = useChatType();
  const { quickBuyAmount, setQuickBuyAmount, agentMode, setAgentMode } =
    useSettings();

  const { t } = useTranslation();

  // Handle quick buy amount change
  const handleAmountChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    setQuickBuyAmount(value);
  };

  // Handle agent mode toggle
  const handleAgentModeToggle = () => {
    setAgentMode(!agentMode);
  };

  return (
    <div className="h-full overflow-auto px-4 scrollable-container">
      <h2 className="text-lg font-bold mb-2 mt-4">
        {t("settings.wallet_addresses")}
      </h2>
      <WalletAddresses />

      <h2 className="text-lg font-bold mb-2 mt-4">{t("settings.quick_buy")}</h2>
      <div className="bg-black/40 backdrop-blur-sm rounded-lg p-4 mb-4">
        <label className="block text-sm text-white mb-2">
          {t("settings.quick_buy_default_sol_amount")}
        </label>
        <div className="flex items-center">
          <input
            type="number"
            value={quickBuyAmount}
            onChange={handleAmountChange}
            min="0.01"
            step="0.01"
            className="bg-black/60 rounded-lg px-3 py-2 text-white w-24 focus:outline-none"
          />
          <span className="ml-2 text-white">SOL</span>
        </div>
        <p className="text-xs text-gray-400 mt-2">
          {t("settings.quick_buy_default_sol_amount_description")}
        </p>
      </div>

      <div className="flex items-center justify-between">
        <h2 className="text-lg font-bold mb-2 mt-4">Agent Mode</h2>
        <button
          onClick={handleAgentModeToggle}
          className={`px-4 py-2 rounded-lg transition-colors ${
            agentMode
              ? "bg-green-500/30 text-green-300 hover:bg-green-500/40"
              : "bg-gray-600/30 text-gray-400 hover:bg-gray-600/40"
          }`}
        >
          {agentMode ? "Enabled" : "Disabled"}
        </button>
      </div>
      <p className="text-xs text-gray-400 mt-2">
        Enabled: Listen will have a lot more autonomy and will execute trades
        directly without any confirmation.{" "}
      </p>
      <p className="text-xs text-gray-400 mt-2">
        Disabled: Listen doesn't have access to direct swapping tools and every
        trade is confirmed by hand.
      </p>

      <h2 className="text-lg font-bold mb-2 mt-4">{t("settings.mode")}</h2>
      <ChatSelector selectedChat={chatType} onSelectChat={setChatType} />

      <h2 className="text-lg font-bold mb-2 mt-4">
        {t("settings.connected_accounts")}
      </h2>
      {user && <ConnectedAccounts user={user} />}
    </div>
  );
}
