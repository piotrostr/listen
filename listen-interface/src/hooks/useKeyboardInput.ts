import { useCallback, useEffect, useState } from "react";

interface UseKeyboardInputProps {
  onSubmit: (text: string) => void;
  onClear: () => void;
  isDisabled?: boolean;
}

export function useKeyboardInput({
  onSubmit,
  onClear,
  isDisabled = false,
}: UseKeyboardInputProps) {
  const [inputMessage, setInputMessage] = useState("");

  const handleKeyPress = useCallback(
    async (e: KeyboardEvent) => {
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      // Handle paste with cmd/ctrl + v
      if ((e.metaKey || e.ctrlKey) && e.key === "v") {
        try {
          const text = await navigator.clipboard.readText();
          setInputMessage((prev) => prev + text);
          e.preventDefault();
          return;
        } catch (err) {
          console.error("Failed to read clipboard:", err);
        }
      }

      // Handle cmd/ctrl + backspace to clear entire text
      if ((e.metaKey || e.ctrlKey) && e.key === "Backspace") {
        setInputMessage("");
        e.preventDefault();
        return;
      }

      if (e.key === "Enter") {
        if (inputMessage.trim() === "clear") {
          onClear();
          setInputMessage("");
          return;
        }
        if (inputMessage.trim()) {
          onSubmit(inputMessage);
          setInputMessage("");
        }
      } else if (e.key === "Backspace") {
        setInputMessage((prev) => prev.slice(0, -1));
      } else if (e.key.length === 1) {
        setInputMessage((prev) => prev + e.key);
      }
    },
    [inputMessage, onSubmit, onClear]
  );

  useEffect(() => {
    if (isDisabled) return;

    window.addEventListener("keydown", handleKeyPress);
    return () => window.removeEventListener("keydown", handleKeyPress);
  }, [handleKeyPress, isDisabled]);

  return { inputMessage, setInputMessage };
}
