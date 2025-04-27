import { Outlet, createRootRoute } from "@tanstack/react-router";
import { Layout } from "../components/Layout";
import { ModalContainer } from "../components/ModalContainer";
import { ChatProvider } from "../contexts/ChatContext";
import { ModalProvider } from "../contexts/ModalContext";

export const Route = createRootRoute({
  component: () => (
    <ChatProvider>
      <ModalProvider>
        <Layout>
          <Outlet />
        </Layout>
        <ModalContainer />
      </ModalProvider>
    </ChatProvider>
  ),
});
