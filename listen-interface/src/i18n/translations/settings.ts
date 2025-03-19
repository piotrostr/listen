export const settings = {
  en: {
    title: "Settings",
    quick_buy: "Quick Buy",
    quick_buy_default_sol_amount: "Default SOL amount for quick buy",
    quick_buy_default_sol_amount_description:
      "This is the default amount of SOL that will be used for quick buys",
    mode: "Mode",
    wallet_addresses: "Wallet addresses",
    connected_accounts: "Connected accounts",
    agent_mode: "Agent Mode",
    enabled: "Enabled",
    agent_mode_enabled: `Listen will have a lot more autonomy and can execute
    direct trades without confirmation. It doesn't have pipelines enabled, but
    it has access to sequential order scheduling tools.`,
    disabled: "Disabled",
    agent_mode_disabled:
      "Every trade is confirmed by hand. Listen doesn't have access to direct swapping tools.",
  },
  zh: {
    title: "设置",
    quick_buy: "快速购买",
    quick_buy_default_sol_amount: "快速购买默认 SOL 数量",
    quick_buy_default_sol_amount_description: "快速购买功能使用的默认 SOL 数量",
    mode: "模式",
    wallet_addresses: "钱包地址",
    connected_accounts: "已连接账户",
    agent_mode: "代理模式",
    enabled: "启用",
    agent_mode_enabled: `Listen 将拥有更多的自主权，并直接执行交易，无需任何确认。
    Listen 没有启用管道，但可以安排顺序订单。`,
    disabled: "禁用",
    agent_mode_disabled:
      "每次交易都需要手动确认。Listen 没有访问直接交换工具的权限。",
  },
};
