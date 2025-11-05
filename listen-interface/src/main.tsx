import "@fontsource/dm-sans/400.css";
import "@fontsource/dm-sans/500.css";
import "@fontsource/dm-sans/700.css";
import "@fontsource/space-grotesk/300.css";
import "@fontsource/space-grotesk/400.css";
import "@fontsource/space-grotesk/500.css";
import "@fontsource/space-grotesk/600.css";
import "@fontsource/space-grotesk/700.css";
import { PrivyProvider } from "@privy-io/react-auth";
import { toSolanaWalletConnectors } from "@privy-io/react-auth/solana";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import { WagmiProvider, createConfig } from "@privy-io/wagmi";
import { http } from "wagmi";
import { arbitrum } from "wagmi/chains";
import { KeyboardProvider } from "./contexts/KeyboardContext";
import { MobileProvider } from "./contexts/MobileContext";
import { SidebarProvider } from "./contexts/SidebarContext";
import { ToastProvider } from "./contexts/ToastContext";

import i18n from "./i18n";
import "./index.css";

// Import the generated route tree
import { routeTree } from "./routeTree.gen";

const config = createConfig({
  chains: [arbitrum],
  transports: {
    [arbitrum.id]: http(),
  },
} as any);

// Create a new router instance
const router = createRouter({ routeTree });

// Register the router instance for type safety
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <PrivyProvider
      appId={"cm6c7ifqd00ar52m1qxfgbkkn"}
      config={{
        // EOA-only mode: disable embedded wallet creation
        embeddedWallets: {
          createOnLogin: 'off',
        },
        // Only allow external wallet connections
        loginMethods: ['wallet'],
        appearance: {
          theme: "dark",
          walletChainType: "ethereum-and-solana",
          showWalletLoginFirst: true,
          walletList: [
            "phantom",
            "okx_wallet",
            "metamask",
            "bybit_wallet",
            "coinbase_wallet",
            "rainbow",
            "wallet_connect",
            "rabby_wallet",
            "solflare",
          ],
        },
        fundingMethodConfig: {
          moonpay: {
            paymentMethod: "credit_debit_card",
          },
        },
        externalWallets: {
          solana: {
            connectors: toSolanaWalletConnectors({
              shouldAutoConnect: true,
            }),
          },
        },
      }}
    >
      <MobileProvider>
        <I18nextProvider i18n={i18n}>
          <ToastProvider>
            <QueryClientProvider client={new QueryClient()}>
              <WagmiProvider config={config}>
                <SidebarProvider>
                  <KeyboardProvider>
                    <RouterProvider router={router} />
                  </KeyboardProvider>
                </SidebarProvider>
              </WagmiProvider>
            </QueryClientProvider>
          </ToastProvider>
        </I18nextProvider>
      </MobileProvider>
    </PrivyProvider>
  </StrictMode>,
);
