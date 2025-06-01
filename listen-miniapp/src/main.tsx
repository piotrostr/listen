import "@fontsource/dm-sans/400.css";
import "@fontsource/dm-sans/500.css";
import "@fontsource/dm-sans/700.css";
import "@fontsource/space-grotesk/300.css";
import "@fontsource/space-grotesk/400.css";
import "@fontsource/space-grotesk/500.css";
import "@fontsource/space-grotesk/600.css";
import "@fontsource/space-grotesk/700.css";
import "@fontsource/work-sans/400.css";
import "@fontsource/work-sans/500.css";
import "@fontsource/work-sans/600.css";
import "@fontsource/work-sans/700.css";
import { PrivyProvider } from "@privy-io/react-auth";
import { toSolanaWalletConnectors } from "@privy-io/react-auth/solana";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import { MiniKitProvider } from "@worldcoin/minikit-js/minikit-provider";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import { worldchain } from "viem/chains";
import { WagmiProvider, createConfig, http } from "wagmi";
import { KeyboardProvider } from "./contexts/KeyboardContext";
import { MobileProvider } from "./contexts/MobileContext";
import { SidebarProvider } from "./contexts/SidebarContext";
import { ToastProvider } from "./contexts/ToastContext";

import i18n from "./i18n";
import "./index.css";

// Import the generated route tree
import { worldchainEnabled } from "./config/env";
import { routeTree } from "./routeTree.gen";

if (window.location.href.includes("staging")) {
  import("eruda")
    .then((eruda) => {
      console.log("Eruda loaded successfully");
      eruda.default.init();
      console.log("Eruda initialized");
    })
    .catch((error) => {
      console.error("Failed to load eruda:", error);
    });
}

const config = createConfig({
  chains: [worldchain],
  transports: {
    [worldchain.id]: http(),
  },
});

// Create a new router instance
const router = createRouter({ routeTree });

// Register the router instance for type safety
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const AppContent = () => {
  const content = (
    <MobileProvider>
      <I18nextProvider i18n={i18n}>
        <PrivyProvider
          appId={"cm6c7ifqd00ar52m1qxfgbkkn"}
          config={{
            appearance: {
              theme: "dark",
              walletChainType: "ethereum-and-solana",
              walletList: [
                "phantom",
                "okx_wallet",
                "metamask",
                "bybit_wallet",
                "coinbase_wallet",
                "rainbow",
                "wallet_connect",
                "rabby_wallet",
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
          <ToastProvider>
            <WagmiProvider config={config}>
              <QueryClientProvider client={new QueryClient()}>
                <SidebarProvider>
                  <KeyboardProvider>
                    <RouterProvider router={router} />
                  </KeyboardProvider>
                </SidebarProvider>
              </QueryClientProvider>
            </WagmiProvider>
          </ToastProvider>
        </PrivyProvider>
      </I18nextProvider>
    </MobileProvider>
  );

  return worldchainEnabled ? (
    <MiniKitProvider>{content}</MiniKitProvider>
  ) : (
    content
  );
};

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <AppContent />
  </StrictMode>
);
