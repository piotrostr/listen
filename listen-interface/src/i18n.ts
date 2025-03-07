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
        chat_history: "Chat History",
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
      share_modal: {
        share_this_chat: "Share this chat",
        anyone_with_this_link_can_view_this_chat:
          "Anyone with this link can view this chat",
        copy: "Copy",
        close: "Close",
        open_in_new_tab: "Open in new tab",
      },
      settings: {
        title: "Settings",
        quick_buy: "Quick Buy",
        quick_buy_default_sol_amount: "Default SOL amount for quick buy",
        quick_buy_default_sol_amount_description:
          "This is the default amount of SOL that will be used for quick buys",
        mode: "Mode",
        wallet_addresses: "Wallet addresses",
        connected_accounts: "Connected accounts",
      },
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
      getting_started: {
        how_it_works: "如何运作",
        how_it_works_description:
          "Listen 是您使用自然语言进行链上交易的一站式平台",
        step_1: "1. 创建账户（可使用邮箱或钱包）",
        step_2: "2. 为 AI 代理初始化钱包，存入资金并授权访问",
        step_3: "3. 开始畅享！",
        get_started: "开始",
        questions:
          "如有疑问可直接询问 Listen，它了解可用工具并管理您的投资组合",
        warning: "Listen 处于早期测试阶段，功能可能不稳定，使用风险自负",
      },
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
      },
      chat: {
        start_a_conversation: "开始对话",
        recommended_questions: {
          what_actions_can_you_perform_for_me: "你能执行哪些操作？",
          how_do_pipelines_work_and_what_pipelines_can_you_create_for_me:
            "自动化任务如何运作，你能为我创建哪些自动化任务？",
          what_chains_are_supported: "支持哪些链？",
          what_tokens_have_received_largest_inflows_outflows_in_the_past_days:
            "过去 24 小时资金流入/流出最多的代币有哪些？",
        },
        placeholder: "输入消息...",
      },
      share_modal: {
        share_this_chat: "分享此聊天",
        anyone_with_this_link_can_view_this_chat:
          "任何人都可以通过此链接查看此聊天",
        copy: "复制",
        close: "关闭",
        open_in_new_tab: "在新标签页中打开",
      },
      settings: {
        title: "设置",
        quick_buy: "快速购买",
        quick_buy_default_sol_amount: "快速购买默认 SOL 数量",
        quick_buy_default_sol_amount_description:
          "快速购买功能使用的默认 SOL 数量",
        mode: "模式",
        wallet_addresses: "钱包地址",
        connected_accounts: "已连接账户",
      },
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
