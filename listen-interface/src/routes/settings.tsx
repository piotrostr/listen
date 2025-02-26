import { usePrivy } from "@privy-io/react-auth";
import { createFileRoute } from "@tanstack/react-router";
import { ChatSelector } from "../components/ChatSelector";
import { ConnectedAccounts } from "../components/ConnectedAccounts";
import { WalletAddresses } from "../components/WalletAddresses";
import { useChatType } from "../hooks/useChatType";

export const Route = createFileRoute("/settings")({
  component: Settings,
});

function Settings() {
  const { user } = usePrivy();
  const { chatType, setChatType } = useChatType();

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">Settings</h1>
      <h2 className="text-lg font-bold mb-2 mt-4">Mode</h2>
      <ChatSelector selectedChat={chatType} onSelectChat={setChatType} />
      <h2 className="text-lg font-bold mb-2 mt-4">Wallet Addresses</h2>
      <WalletAddresses />
      <h2 className="text-lg font-bold mb-2 mt-4">Connected Accounts</h2>
      {user && <ConnectedAccounts user={user} />}
    </div>
  );
}
