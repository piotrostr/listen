import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { PrivyProvider, usePrivy } from "@privy-io/react-auth";
import { GettingStarted } from "./components/GettingStarted";
import { Layout } from "./components/Layout";
import {
  userHasDelegatedEvmWallet,
  userHasDelegatedSolanaWallet,
} from "./hooks/util";
import { LoggedInView } from "./components/LoggedInView";
import { createConfig, http, WagmiProvider } from "wagmi";
import { arbitrum } from "viem/chains";

const config = createConfig({
  chains: [arbitrum],
  transports: {
    [arbitrum.id]: http(),
  },
});

function App() {
  return (
    <PrivyProvider appId={"cm6c7ifqd00ar52m1qxfgbkkn"} config={{}}>
      <WagmiProvider config={config}>
        <QueryClientProvider client={new QueryClient()}>
          <Inner />
        </QueryClientProvider>
      </WagmiProvider>
    </PrivyProvider>
  );
}

function Inner() {
  const { authenticated, ready, user } = usePrivy();
  const isDelegatedSolana = userHasDelegatedSolanaWallet(user);
  const isDelegatedEvm = userHasDelegatedEvmWallet(user);

  if (!ready) {
    return (
      <Layout>
        <></>
      </Layout>
    );
  }
  return (
    <Layout>
      {ready && authenticated && isDelegatedSolana && isDelegatedEvm ? (
        <LoggedInView />
      ) : (
        <GettingStarted />
      )}
    </Layout>
  );
}

export default App;
