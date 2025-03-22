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
import { arbitrum } from "viem/chains";
import { WagmiProvider, createConfig, http } from "wagmi";
import { MobileProvider } from "./contexts/MobileContext";
import { ModalProvider } from "./contexts/ModalContext";
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
});

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
                  <ModalProvider>
                    <RouterProvider router={router} />
                  </ModalProvider>
                </SidebarProvider>
              </QueryClientProvider>
            </WagmiProvider>
          </ToastProvider>
        </PrivyProvider>
      </I18nextProvider>
    </MobileProvider>
  </StrictMode>
);
