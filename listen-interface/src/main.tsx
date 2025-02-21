import { PrivyProvider } from "@privy-io/react-auth";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { arbitrum } from "viem/chains";
import { WagmiProvider, createConfig, http } from "wagmi";
import { ModalProvider } from "./contexts/ModalContext";
import { ToastProvider } from "./contexts/ToastContext";
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
    <PrivyProvider appId={"cm6c7ifqd00ar52m1qxfgbkkn"} config={{}}>
      <ToastProvider>
        <WagmiProvider config={config}>
          <QueryClientProvider client={new QueryClient()}>
            <ModalProvider>
              <RouterProvider router={router} />
            </ModalProvider>
          </QueryClientProvider>
        </WagmiProvider>
      </ToastProvider>
    </PrivyProvider>
  </StrictMode>
);
