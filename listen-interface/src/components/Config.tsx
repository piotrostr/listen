import { usePrivy } from "@privy-io/react-auth";
import { DelegateActionButton } from "./DelegateActionButton";
import { useEffect, useState } from "react";
import { config } from "../config";

export function Config() {
  const { getAccessToken, unlinkEmail, user, linkWallet, logout } = usePrivy();
  const [accessToken, setAccessToken] = useState<string | null>(null);

  useEffect(() => {
    if (!accessToken)
      getAccessToken().then((token) => {
        setAccessToken(token);
      });
  }, [accessToken, getAccessToken]);

  const fetchAuth = async () => {
    const res = await fetch(config.API_BASE_URL + "/v1/auth", {
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
    const res = await fetch(config.API_BASE_URL + "/v1/test_tx", {
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        Authorization: `Bearer ${await getAccessToken()}`,
      },
    });
    const data = await res.text();
    console.log(res.status, data);
  };

  const fetchBalanceTx = async () => {
    const res = await fetch(config.API_BASE_URL + "/v1/test_balance", {
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        Authorization: `Bearer ${await getAccessToken()}`,
      },
    });
    const data = await res.text();
    console.log(res.status, data);
  };

  const handleClickCopy = () => {
    if (!accessToken) return;
    navigator.clipboard.writeText(accessToken);
  };

  const handleClickTestBalance = async () => {
    for (let i = 0; i < 10; i++) {
      // sleep for 200 millis
      await Promise.resolve(setTimeout(() => {}, 200));
      await fetchBalanceTx();
    }
  };

  return (
    <div className="flex flex-row gap-4 p-4 justify-center">
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
      <button onClick={handleClickCopy} className="btn">
        CopyAccessToken
      </button>
      <button onClick={handleClickTestBalance} className="btn">
        TestBalance
      </button>
    </div>
  );
}
