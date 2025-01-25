import { Background } from "./components/Background";
import { Chat } from "./components/Chat";
import { Header } from "./components/Header";
import { Portfolio } from "./components/Portfolio";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { PrivyProvider, usePrivy } from "@privy-io/react-auth";

const queryClient = new QueryClient();

function App() {
  const privyAppId = import.meta.env.VITE_PRIVY_APP_ID;
  return (
    <PrivyProvider appId={privyAppId}>
      <QueryClientProvider client={queryClient}>
        <Contents />
      </QueryClientProvider>
    </PrivyProvider>
  );
}

function Contents() {
  const { login, ready, authenticated } = usePrivy();
  return (
    <div className="relative min-h-screen text-white">
      <Background />
      <Header />
      <div className="relative z-10 py-20">
        <div className="flex flex-col lg:flex-row gap-4 max-w-7xl mx-auto px-4">
          <div className="flex-1">
            {authenticated ? (
              <Chat />
            ) : (
              <button onClick={login} disabled={!ready} className="btn">
                Login
              </button>
            )}
          </div>
          <div className="w-full lg:w-80">
            <Portfolio />
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
