import { useCallback, useEffect, useState } from "react";

interface UseKeyboardInputProps {
  onSubmit: (text: string) => void;
  onClear: () => void;
  onStopGeneration: () => void;
  isGenerating: boolean;
  isDisabled?: boolean;
}

export function useKeyboardInput({
  onSubmit,
  onClear,
  onStopGeneration,
  isGenerating,
  isDisabled = false,
}: UseKeyboardInputProps) {
  const [inputMessage, setInputMessage] = useState("");
  const [history, setHistory] = useState<string[]>([]);
  const [_historyIndex, setHistoryIndex] = useState(-1);

  // Submit the current message
  const submitMessage = useCallback(() => {
    if (inputMessage.trim()) {
      if (inputMessage.trim() === "clear") {
        onClear();
      } else {
        onSubmit(inputMessage);
        // Add to history
        setHistory((prev) => [inputMessage, ...prev.slice(0, 19)]);
      }
      setInputMessage("");
      setHistoryIndex(-1);
    }
  }, [inputMessage, onSubmit, onClear]);

  // Handle keyboard navigation
  const handleKeyPress = useCallback(
    async (e: KeyboardEvent) => {
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      // Stop generation with Escape key
      if (e.key === "Escape" && isGenerating) {
        onStopGeneration();
        e.preventDefault();
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

      // Handle history navigation with up/down arrows
      if (e.key === "ArrowUp") {
        setHistoryIndex((prev) => {
          const newIndex = Math.min(prev + 1, history.length - 1);
          if (newIndex >= 0 && history[newIndex]) {
            setInputMessage(history[newIndex]);
          }
          return newIndex;
        });
        e.preventDefault();
        return;
      }

      if (e.key === "ArrowDown") {
        setHistoryIndex((prev) => {
          const newIndex = Math.max(prev - 1, -1);
          if (newIndex >= 0) {
            setInputMessage(history[newIndex]);
          } else {
            setInputMessage("");
          }
          return newIndex;
        });
        e.preventDefault();
        return;
      }

      if (e.key === "Enter") {
        if (isGenerating) {
          onStopGeneration();
        } else {
          submitMessage();
        }
        e.preventDefault();
      } else if (e.key === "Backspace") {
        setInputMessage((prev) => prev.slice(0, -1));
      } else if (e.key.length === 1) {
        setInputMessage((prev) => prev + e.key);
      }
    },
    [inputMessage, submitMessage, onStopGeneration, isGenerating, history]
  );

  useEffect(() => {
    if (isDisabled) return;

    window.addEventListener("keydown", handleKeyPress);
    return () => window.removeEventListener("keydown", handleKeyPress);
  }, [handleKeyPress, isDisabled]);

  return {
    inputMessage,
    setInputMessage,
    submitMessage,
  };
}
