import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { gettingStarted } from "./translations/getting-started";
import { layout } from "./translations/layout";
import { pipelines } from "./translations/pipelines";
import { priceUpdates } from "./translations/price-updates";
import { recommendedQuestions } from "./translations/recommended-questions";
import { settings } from "./translations/settings";
import { toolCalls } from "./translations/tool-calls";
import { toolMessages } from "./translations/tool-messages";

const resources = {
  en: {
    translation: {
      tool_calls: toolCalls.en,
      tool_messages: toolMessages.en,
      getting_started: gettingStarted.en,
      layout: layout.en,
      chat_history: {
        no_chats_found: "No chats found",
      },
      recent_chats: {
        view_all_chats: "View all chats",
      },
      pipelines: pipelines.en,
      token_tile: {
        traders: "traders",
        market_cap: "MC",
        executing: "Buying...",
      },
      pipeline_execution: {
        pipeline_scheduled: "Pipeline scheduled for execution",
        error: "Error occured",
        buy_order_placed: "Buy order placed for",
        failed_to_buy_token: "Failed to buy token",
        cancelled: "Cancelled",
      },
      price_updates: priceUpdates.en,
      chat: {
        start_a_conversation: "Start a conversation",
        placeholder: "How can I help?",
      },
      recommended_questions: recommendedQuestions.en,
      share_modal: {
        share: "Share",
        rename: "Rename",
        delete: "Delete",
        share_this_chat: "Share this chat",
        anyone_with_this_link_can_view_this_chat:
          "Anyone with this link can view this chat",
        copy: "Copy",
        close: "Close",
        open_in_new_tab: "Open in new tab",
      },
      settings: settings.en,
      wallet_addresses: {
        solana_wallet: "Solana Wallet",
        evm_wallet: "EVM Wallet",
        export: "Export",
        fund: "Fund",
      },
      portfolio: {
        title: "Portfolio",
        holding: "Holding",
        buy: "Buy",
        sell: "Sell",
        no_assets_found: "No assets found",
      },
      buy_sell_modal: {
        available: "Available",
        processing: "Processing...",
        buy: "Buy",
        sell: "Sell",
        amount: "Amount",
      },
    },
  },
  zh: {
    translation: {
      tool_calls: toolCalls.zh,
      getting_started: gettingStarted.zh,
      layout: layout.zh,
      chat_history: {
        new_chat: "新聊天",
      },
      recent_chats: {
        view_all_chats: "查看所有聊天",
      },
      pipelines: pipelines.zh,
      token_tile: {
        traders: "交易者",
        market_cap: "市值",
        executing: "购买中...",
      },
      pipeline_execution: {
        pipeline_scheduled: "自动化任务已安排执行",
        error: "发生错误",
        buy_order_placed: "已为以下代币下单购买",
        failed_to_buy_token: "购买代币失败",
        cancelled: "已取消",
      },
      buy_sell_modal: {
        available: "可用",
        processing: "处理中...",
        buy: "购买",
        sell: "出售",
        amount: "金额",
      },
      price_updates: priceUpdates.zh,
      chat: {
        start_a_conversation: "开始对话",
        placeholder: "我能为你做什么？",
      },
      recommended_questions: recommendedQuestions.zh,
      share_modal: {
        share_this_chat: "分享此聊天",
        anyone_with_this_link_can_view_this_chat:
          "任何人都可以通过此链接查看此聊天",
        copy: "复制",
        close: "关闭",
        open_in_new_tab: "在新标签页中打开",
        share: "分享",
        rename: "重命名",
        delete: "删除",
      },
      settings: settings.zh,
      wallet_addresses: {
        solana_wallet: "Solana 钱包",
        evm_wallet: "EVM 钱包",
        export: "导出",
        fund: "资金",
      },
      portfolio: {
        title: "投资组合",
        holding: "持有资产",
        buy: "购买",
        sell: "出售",
        no_assets_found: "未找到资产",
      },
      tool_messages: toolMessages.zh,
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
