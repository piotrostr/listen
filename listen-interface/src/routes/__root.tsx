import { Outlet, createRootRoute } from "@tanstack/react-router";
import { Layout } from "../components/Layout";
import { ModalContainer } from "../components/ModalContainer";
import { ChatProvider } from "../contexts/ChatContext";
import { ModalProvider } from "../contexts/ModalContext";
import { PanelProvider } from "../contexts/PanelContext";

function ErrorComponent() {
  return (
    <div className="flex items-center justify-center h-screen bg-black text-white">
      <div className="text-center p-8 rounded-lg">
        <h1 className="text-2xl font-bold mb-4">Oops! Something went wrong</h1>
        <p className="mb-4">We encountered an error while loading this page.</p>
        <a
          href="/"
          className="px-4 py-2 bg-[#2D2D2D] hover:bg-[#3D3D3D] rounded-lg transition-colors"
        >
          Return Home
        </a>
      </div>
    </div>
  );
}

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
  errorComponent: ErrorComponent,
});
