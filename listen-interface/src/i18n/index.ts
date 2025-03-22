import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { buySellModal } from "./translations/buy-sell-modal";
import { chat } from "./translations/chat";
import { chatHistory } from "./translations/chat-history";
import { gettingStarted } from "./translations/getting-started";
import { layout } from "./translations/layout";
import { pipelineExecution } from "./translations/pipeline-execution";
import { pipelines } from "./translations/pipelines";
import { portfolio } from "./translations/portfolio";
import { priceUpdates } from "./translations/price-updates";
import { recentChats } from "./translations/recent-chats";
import { recommendedQuestions } from "./translations/recommended-questions";
import { settings } from "./translations/settings";
import { shareModal } from "./translations/share-modal";
import { tokenTile } from "./translations/token-tile";
import { toolCalls } from "./translations/tool-calls";
import { toolMessages } from "./translations/tool-messages";
import { version } from "./translations/version";
import { walletAddresses } from "./translations/wallet-addresses";

const resources = {
  en: {
    translation: {
      version: version.en,
      tool_calls: toolCalls.en,
      tool_messages: toolMessages.en,
      getting_started: gettingStarted.en,
      layout: layout.en,
      chat_history: chatHistory.en,
      recent_chats: recentChats.en,
      pipelines: pipelines.en,
      token_tile: tokenTile.en,
      pipeline_execution: pipelineExecution.en,
      price_updates: priceUpdates.en,
      chat: chat.en,
      recommended_questions: recommendedQuestions.en,
      share_modal: shareModal.en,
      settings: settings.en,
      wallet_addresses: walletAddresses.en,
      portfolio: portfolio.en,
      buy_sell_modal: buySellModal.en,
    },
  },
  zh: {
    translation: {
      version: version.zh,
      tool_calls: toolCalls.zh,
      tool_messages: toolMessages.zh,
      getting_started: gettingStarted.zh,
      layout: layout.zh,
      chat_history: chatHistory.zh,
      recent_chats: recentChats.zh,
      pipelines: pipelines.zh,
      token_tile: tokenTile.zh,
      pipeline_execution: pipelineExecution.zh,
      price_updates: priceUpdates.zh,
      chat: chat.zh,
      recommended_questions: recommendedQuestions.zh,
      share_modal: shareModal.zh,
      settings: settings.zh,
      wallet_addresses: walletAddresses.zh,
      portfolio: portfolio.zh,
      buy_sell_modal: buySellModal.zh,
    },
  },
};

const isChineseLocale = (locale: string) => {
  return locale.startsWith("zh-");
};

// Get user's browser locale
const getBrowserLocale = () => {
  const browserLocale = navigator.language;
  if (isChineseLocale(browserLocale)) {
    return "zh";
  }
  return "en";
};

export const savedLanguage =
  localStorage.getItem("language") || getBrowserLocale();

i18n.use(initReactI18next).init({
  resources,
  lng: savedLanguage,
  fallbackLng: "en",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
