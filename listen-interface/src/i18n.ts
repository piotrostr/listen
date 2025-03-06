import i18n from "i18next";
import { initReactI18next } from "react-i18next";

const resources = {
  en: {
    translation: {
      getting_started: {
        how_it_works: "How it works",
        how_it_works_description:
          "Listen is your single stop for on-chain trading with natural language",
        step_1: "1. create an account, you can use your email or wallet",
        step_2:
          "2. initialize a wallet for your AI agent, deposit funds and delegate access",
        step_3: "3. go wild!",
        get_started: "Get Started",
        questions:
          "Should you have any questions - ask Listen directly, it understands the tools it has access to and has a view of the portfolio its managing",
        warning:
          "Listen is in early beta, things might not work as expected, use at own risk",
      },
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
      },
      price_updates: {
        paused: "Paused",
        market_cap: "Market Cap",
        all: "Any",
        waiting_for_updates: "Waiting for updates...",
      },
      chat: {
        recommended_questions: {
          what_actions_can_you_perform_for_me:
            "What actions can you perform for me?",
          how_do_pipelines_work_and_what_pipelines_can_you_create_for_me:
            "How do pipelines work and what pipelines can you create for me?",
          what_chains_are_supported: "What chains are supported?",
          what_tokens_have_received_largest_inflows_outflows_in_the_past_days:
            "What tokens have received largest inflows/outflows in the past days?",
        },
        start_a_conversation: "Start a conversation",
        placeholder: "Type your message...",
      },
    },
  },
  zh: {
    translation: {
      getting_started: {
        how_it_works: "如何运作",
        how_it_works_description:
          "Listen 是您的单一目的地，用于使用自然语言进行链上交易",
        step_1: "1. 创建一个帐户，您可以使用电子邮件或钱包",
        step_2: "2. 为您的 AI 代理初始化一个钱包，存入资金并委托访问",
        step_3: "3. 尽情享受吧！",
        step_4: "4. 尽情享受吧！",
        get_started: "开始",
        questions:
          "如果您有任何问题 - 直接询问代理 - Listen 了解它拥有的工具并拥有其管理的投资组合的视图",
        warning:
          "Listen 处于早期测试阶段，可能会出现意想不到的问题，请自行承担风险",
      },
      layout: {
        screener: "筛选器",
        portfolio: "投资组合",
        pipelines: "管道",
        settings: "设置",
        documentation: "文档",
        github: "Github",
        twitter: "Twitter",
        logout: "登出",
        chat: "聊天",
      },
      recent_chats: {
        view_all_chats: "查看所有聊天",
      },
      pipelines: {
        pipelines: "管道",
        please_connect_wallet: "请连接您的钱包以继续",
        all: "全部",
        pending: "待处理",
        completed: "已完成",
        failed: "失败",
        no_pipelines_found: "没有找到管道",
        id: "ID",
        created: "创建",
        send_notification: "发送通知",
        conditions: "条件",
        execute_immediately: "立即执行",
        for: "对于",
        price_above: "价格高于",
        price_below: "价格低于",
        status: "状态",
        slippage_tolerance_exceeded: "滑点容忍度超出",
        insufficient_balance: "余额不足",
        cancelled: "已取消",
        amount: "金额",
        pipeline_status: {
          Pending: "待处理",
          Completed: "已完成",
          Failed: "失败",
          Cancelled: "已取消",
        },
        approve: "批准",
        reject: "拒绝",
        pipeline_rejected: "管道已拒绝",
        pipeline_scheduled_for_execution: "管道已安排执行",
      },
      token_tile: {
        traders: "交易者",
        market_cap: "市值",
        executing: "购买中...",
      },
      pipeline_execution: {
        pipeline_scheduled: "管道已安排执行",
        error: "发生错误",
        buy_order_placed: "购买订单已放置",
        failed_to_buy_token: "购买失败",
      },
      price_updates: {
        paused: "暂停",
        market_cap: "市值",
        all: "任何",
        waiting_for_updates: "等待更新...",
      },
      chat: {
        start_a_conversation: "开始对话",
        recommended_questions: {
          what_actions_can_you_perform_for_me: "你能为我做什么？",
          how_do_pipelines_work_and_what_pipelines_can_you_create_for_me:
            "管道如何运作以及你能为我创建什么管道？",
          what_chains_are_supported: "支持哪些链？",
          what_tokens_have_received_largest_inflows_outflows_in_the_past_days:
            "过去几天有哪些代币获得了最大的流入/流出？",
        },
        placeholder: "输入您的消息...",
      },
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
