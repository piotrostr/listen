import { usePrivy } from "@privy-io/react-auth";
import { useTranslation } from "react-i18next";
import { useSettingsStore } from "../store/settingsStore";
import { ChatSelector } from "./ChatSelector";
import { ConnectedAccounts } from "./ConnectedAccounts";
import { WalletAddresses } from "./WalletAddresses";

export function Settings() {
  const { user } = usePrivy();
  const { chatType, setChatType } = useSettingsStore();
  const {
    quickBuyAmount,
    setQuickBuyAmount,
    agentMode,
    setAgentMode,
    debugMode,
    setDebugMode,
  } = useSettingsStore();

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
        <h2 className="text-lg font-bold mb-2 mt-4">
          {t("settings.agent_mode")}
        </h2>
        <button
          onClick={handleAgentModeToggle}
          className={`px-4 py-2 rounded-lg transition-colors ${
            agentMode
              ? "bg-green-500/30 text-green-300 hover:bg-green-500/40"
              : "bg-gray-600/30 text-gray-400 hover:bg-gray-600/40"
          }`}
        >
          {agentMode ? t("settings.enabled") : t("settings.disabled")}
        </button>
      </div>
      <p className="text-xs text-gray-400 mt-2">
        {t("settings.agent_mode_enabled")}
      </p>
      <p className="text-xs text-gray-400 mt-2">
        {t("settings.agent_mode_disabled")}
      </p>

      <h2 className="text-lg font-bold mb-2 mt-4">{t("settings.mode")}</h2>
      <ChatSelector selectedChat={chatType} onSelectChat={setChatType} />

      <h2 className="text-lg font-bold mb-2 mt-4">
        {t("settings.connected_accounts")}
      </h2>
      {user && <ConnectedAccounts user={user} />}
      {process.env.NODE_ENV !== "production" && (
        <button
          onClick={() => {
            setDebugMode(!debugMode);
          }}
        >
          Debug mode: {debugMode ? "enabled" : "disabled"}
        </button>
      )}
    </div>
  );
}
