import { Outlet, createRootRoute } from "@tanstack/react-router";
import { Layout } from "../components/Layout";
import { ModalContainer } from "../components/ModalContainer";
import { ChatProvider } from "../contexts/ChatContext";
import { ModalProvider } from "../contexts/ModalContext";
import { PanelProvider } from "../contexts/PanelContext";

export const Route = createRootRoute({
  component: () => (
    <PanelProvider>
      <ChatProvider>
        <ModalProvider>
          <Layout>
            <Outlet />
          </Layout>
          <ModalContainer />
        </ModalProvider>
      </ChatProvider>
    </PanelProvider>
  ),
});
