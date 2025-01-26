import { useState, useEffect } from "react";
import { useChat } from "../hooks/useChat";
import { ToolOutputDisplay } from "./ToolOutputDisplay";
import { usePrivy } from "@privy-io/react-auth";
import { DelegateActionButton } from "./DelegateActionButton";

export function Chat() {
  const { messages, isLoading, sendMessage, toolOutput } = useChat();
  const [inputMessage, setInputMessage] = useState("");

  const { getAccessToken } = usePrivy();

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

  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      if (e.key === "Enter") {
        if (inputMessage.trim()) {
          sendMessage(inputMessage);
          setInputMessage("");
        }
      } else if (e.key === "Backspace") {
        setInputMessage((prev) => prev.slice(0, -1));
      } else if (e.key.length === 1) {
        setInputMessage((prev) => prev + e.key);
      }
    };

    window.addEventListener("keydown", handleKeyPress);
    return () => window.removeEventListener("keydown", handleKeyPress);
  }, [inputMessage, sendMessage]);

  return (
    <div className="flex flex-col gap-4 h-[70vh] w-full max-w-4xl mx-auto px-4 font-mono">
      <div className="flex-1 overflow-hidden">
        <div className="h-full border-2 border-purple-500/30 rounded-lg overflow-hidden bg-black/40 backdrop-blur-sm">
          <div className="h-full flex flex-col">
            <div className="flex flex-row gap-2 p-4 justify-center">
              <DelegateActionButton />
              <button onClick={fetchAuth} className="btn">
                Auth
              </button>
              <button onClick={fetchTestTx} className="btn">
                TestTx
              </button>
            </div>
            <div className="flex-1 overflow-y-auto p-4 scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent">
              {messages.map(
                (msg) =>
                  msg.message && (
                    <div
                      key={msg.id}
                      className={`
                        ${
                          msg.direction === "incoming"
                            ? "bg-blue-900/20 text-blue-300"
                            : "bg-purple-900/20 text-purple-300"
                        }
                        rounded-lg px-4 py-2 my-2 backdrop-blur-sm
                        border border-opacity-20
                        ${
                          msg.direction === "incoming"
                            ? "border-blue-500"
                            : "border-purple-500"
                        }
                      `}
                    >
                      {msg.message}
                    </div>
                  ),
              )}

              {toolOutput && (
                <div className="bg-blue-900/20 text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500">
                  <ToolOutputDisplay toolOutput={toolOutput} />
                </div>
              )}

              {isLoading && (
                <div className="bg-purple-900/20 text-purple-300 rounded px-4 py-2">
                  ...
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      <div className="h-12 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3">
        <span className="text-white whitespace-pre">{inputMessage}</span>
        <span className="w-2 h-5 bg-white terminal-blink ml-[1px]" />
      </div>
    </div>
  );
}
