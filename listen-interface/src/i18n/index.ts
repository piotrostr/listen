import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { gettingStarted } from "./translations/getting-started";
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
      layout: {
        screener: "Screener",
        portfolio: "Portfolio",
        pipelines: "Pipelines",
        settings: "Settings",
        documentation: "Documentation",
        github: "Github",
        twitter: "Twitter",
        chat: "Chat",
        logout: "Logout",
        chat_history: "Chat History",
        recent_chats: "Recent Chats",
        version: "Version",
        new_chat: "New Chat",
      },
      chat_history: {
        no_chats_found: "No chats found",
      },
      recent_chats: {
        view_all_chats: "View all chats",
      },
      pipelines: {
        pipelines: "Pipelines",
        please_connect_wallet: "Please connect your wallet to continue",
        all: "All",
        pending: "Pending",
        completed: "Completed",
        failed: "Failed",
        no_pipelines_found: "No pipelines found",
        id: "ID",
        created: "Created",
        send_notification: "Send a notification",
        conditions: "Conditions",
        execute_immediately: "Execute immediately",
        for: "for",
        price_above: "Price above",
        price_below: "Price below",
        status: "Status",
        slippage_tolerance_exceeded: "Slippage tolerance exceeded",
        insufficient_balance: "Insufficient balance",
        cancelled: "Cancelled",
        pipeline_status: {
          Pending: "Pending",
          Completed: "Completed",
          Failed: "Failed",
          Cancelled: "Cancelled",
        },
        approve: "Approve",
        reject: "Reject",
        pipeline_rejected: "Pipeline rejected",
        pipeline_scheduled_for_execution: "Pipeline scheduled for execution",
        invalid_timestamp: "Invalid timestamp",
        amount: "Amount",
      },
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
      price_updates: {
        paused: "Paused",
        market_cap: "Market Cap",
        all: "Any",
        filter: "Filter",
        volume: "Volume",
        sell: "Sell",
        waiting_for_updates: "Waiting for updates...",
      },
      chat: {
        start_a_conversation: "Start a conversation",
        placeholder: "How can I help?",
      },
      recommended_questions: recommendedQuestions.en,
      share_modal: {
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
      layout: {
        screener: "市场筛选",
        portfolio: "投资组合",
        pipelines: "自动化任务",
        settings: "设置",
        documentation: "文档",
        github: "Github",
        twitter: "推特",
        chat: "聊天",
        logout: "退出登录",
        chat_history: "聊天记录",
        recent_chats: "最近聊天",
        new_chat: "新聊天",
        no_chats_found: "未找到聊天",
        version: "版本",
      },
      chat_history: {
        new_chat: "新聊天",
      },
      recent_chats: {
        view_all_chats: "查看所有聊天",
      },
      pipelines: {
        pipelines: "自动化任务",
        please_connect_wallet: "请连接钱包以继续",
        all: "全部",
        pending: "处理中",
        completed: "已完成",
        failed: "失败",
        no_pipelines_found: "未找到自动化任务",
        id: "ID",
        created: "创建",
        send_notification: "发送通知",
        conditions: "条件",
        execute_immediately: "立即执行",
        for: "对于",
        price_above: "价格高于",
        price_below: "价格低于",
        status: "状态",
        slippage_tolerance_exceeded: "滑点超出范围",
        insufficient_balance: "余额不足",
        cancelled: "已取消",
        amount: "金额",
        pipeline_status: {
          Pending: "处理中",
          Completed: "已完成",
          Failed: "失败",
          Cancelled: "已取消",
        },
        approve: "批准",
        reject: "拒绝",
        pipeline_rejected: "自动化任务已被拒绝",
        pipeline_scheduled_for_execution: "自动化任务已安排执行",
        invalid_timestamp: "无效的时间戳",
      },
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
      price_updates: {
        paused: "暂停",
        market_cap: "市值",
        all: "全部",
        waiting_for_updates: "等待更新...",
        filter: "过滤",
        volume: "成交量",
        sell: "出售",
      },
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
