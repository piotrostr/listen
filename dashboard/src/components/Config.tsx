import { usePrivy } from "@privy-io/react-auth";
import { DelegateActionButton } from "./DelegateActionButton";

export function Config() {
  const { getAccessToken, unlinkEmail, user, linkWallet, logout } = usePrivy();

  const fetchAuth = async () => {
    const res = await fetch("http://localhost:8080/v1/auth", {
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        Authorization: `Bearer ${await getAccessToken()}`,
      },
    });
    const data = await res.text();
    console.log(res.status, data);
  };

  const fetchTestTx = async () => {
    const res = await fetch("http://localhost:8080/v1/test_tx", {
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        Authorization: `Bearer ${await getAccessToken()}`,
      },
    });
    const data = await res.text();
    console.log(res.status, data);
  };

  return (
    <div className="flex flex-row gap-2 p-4 justify-center">
      <DelegateActionButton />
      <button onClick={fetchAuth} className="btn">
        Auth
      </button>
      <button onClick={fetchTestTx} className="btn">
        TestTx
      </button>
      <button
        onClick={() => {
          if (user?.email?.address) unlinkEmail(user?.email?.address);
        }}
        className="btn"
      >
        UnlinkEmail
      </button>
      <button onClick={linkWallet} className="btn">
        LinkWallet
      </button>
      <button onClick={logout} className="btn">
        Logout
      </button>
    </div>
  );
}
