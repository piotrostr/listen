import { Outlet, createRootRoute } from "@tanstack/react-router";
import { Layout } from "../components/Layout";
import { ChatProvider } from "../contexts/ChatContext";

export const Route = createRootRoute({
  component: () => (
    <ChatProvider>
      <Layout>
        <Outlet />
      </Layout>
    </ChatProvider>
  ),
});
