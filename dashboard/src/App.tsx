import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  PrivyProvider,
  PrivyProviderProps,
  usePrivy,
} from "@privy-io/react-auth";
import { GettingStarted } from "./components/GettingStarted";
import { LoggedInView } from "./components/LoggedInView";
import { Layout } from "./components/Layout";

const queryClient = new QueryClient();

const privyConfig: PrivyProviderProps["config"] = {
  appearance: {
    // walletChainType: "solana-only",
  },
};

function App() {
  const privyAppId = import.meta.env.VITE_PRIVY_APP_ID;
  return (
    <PrivyProvider appId={privyAppId} config={privyConfig}>
      <QueryClientProvider client={queryClient}>
        <Inner />
      </QueryClientProvider>
    </PrivyProvider>
  );
}

function Inner() {
  const { authenticated, ready } = usePrivy();
  if (!ready) {
    return (
      <Layout>
        <></>
      </Layout>
    );
  }
  return (
    <Layout>
      {ready && authenticated ? <LoggedInView /> : <GettingStarted />}
    </Layout>
  );
}

export default App;
