import { usePrivy, User } from "@privy-io/react-auth";
import { FaEnvelope, FaGoogle, FaPhone, FaXTwitter } from "react-icons/fa6";
import { imageMap } from "../hooks/util";
import { ConnectedAccount } from "./ConnectedAccount";

interface ConnectedAccountsProps {
  user: User;
}

export function ConnectedAccounts({ user }: ConnectedAccountsProps) {
  const { linkEmail, linkGoogle, linkPhone, linkTwitter, connectWallet } =
    usePrivy();

  const accounts = [
    {
      icon: (
        <img src={imageMap.eth} alt="ETH" className="w-4 h-4 rounded-full" />
      ),
      isConnected: !!user.wallet,
      onConnect: connectWallet,
      value: user.wallet?.address || "",
    },
    {
      icon: <FaXTwitter className="w-4 h-4" />,
      isConnected: !!user.twitter,
      onConnect: linkTwitter,
      value: user.twitter?.username || "",
    },
    {
      icon: <FaGoogle className="w-4 h-4" />,
      isConnected: !!user.google,
      onConnect: linkGoogle,
      value: user.google?.email || "",
    },
    {
      icon: <FaEnvelope className="w-4 h-4" />,
      isConnected: !!user.email,
      onConnect: linkEmail,
      value: user.email?.address || "",
    },
    {
      icon: <FaPhone className="w-4 h-4" />,
      isConnected: !!user.phone,
      onConnect: linkPhone,
      value: user.phone?.number || "",
    },
  ];

  return (
    <div>
      <div className="border border-purple-500/30 rounded-lg p-4 bg-black/40 backdrop-blur-sm">
        <div className="flex flex-col gap-2">
          <div className="flex flex-col gap-2">
            {accounts
              .filter((account) => account.isConnected)
              .map((account, i) => (
                <ConnectedAccount key={i} {...account} />
              ))}
          </div>

          <div className="flex flex-row gap-2 flex-wrap justify-center">
            {accounts
              .filter((account) => !account.isConnected)
              .map((account, i) => (
                <ConnectedAccount key={i} {...account} />
              ))}
          </div>
        </div>
      </div>
    </div>
  );
}
