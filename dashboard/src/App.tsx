import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  PrivyProvider,
  PrivyProviderProps,
  usePrivy,
} from "@privy-io/react-auth";
import { GettingStarted } from "./components/GettingStarted";
import { Layout } from "./components/Layout";
import { userHasDelegatedWallet } from "./hooks/util";
import { LoggedInView } from "./components/LoggedInView";

const queryClient = new QueryClient();

const privyConfig: PrivyProviderProps["config"] = {
  appearance: {
    // walletChainType: "solana-only",
  },
};

function App() {
  return (
    <PrivyProvider appId={"cm6c7ifqd00ar52m1qxfgbkkn"} config={privyConfig}>
      <QueryClientProvider client={queryClient}>
        <Inner />
      </QueryClientProvider>
    </PrivyProvider>
  );
}

function Inner() {
  const { authenticated, ready, user } = usePrivy();
  const isDelegated = userHasDelegatedWallet(user);

  if (!ready) {
    return (
      <Layout>
        <></>
      </Layout>
    );
  }
  return (
    <Layout>
      {ready && authenticated && isDelegated ? (
        <LoggedInView />
      ) : (
        <GettingStarted />
      )}
    </Layout>
  );
}

export default App;
