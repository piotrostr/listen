import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { FiSend } from "react-icons/fi";
import { RiEdit2Line } from "react-icons/ri";
import { useChat } from "../contexts/ChatContext";
import { Message } from "../types/message";
import { ChatMessage } from "./ChatMessage";

export const EditableMessage = ({
  message,
  isLastUserMessage = false,
}: {
  message: Message;
  isLastUserMessage: boolean;
}) => {
  const { t } = useTranslation();
  const { editMessage, resendMessage } = useChat();
  const [isEditing, setIsEditing] = useState(false);
  const [editedContent, setEditedContent] = useState(message.message);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (isEditing && textareaRef.current) {
      textareaRef.current.focus();
      textareaRef.current.setSelectionRange(
        textareaRef.current.value.length,
        textareaRef.current.value.length
      );
    }
  }, [isEditing]);

  // Auto-resize the textarea as content changes
  useEffect(() => {
    if (isEditing && textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
    }
  }, [editedContent, isEditing]);

  const handleSave = async () => {
    if (editedContent.trim() === "") return;

    // First, update the message in the UI
    editMessage(message.id, editedContent);
    setIsEditing(false);

    // Then resend with the updated content
    await resendMessage(message.id, editedContent);
  };

  if (!isEditing) {
    return (
      <div className="flex w-full justify-end">
        <div className="relative group inline-block">
          <ChatMessage
            message={message.message}
            direction={message.direction}
          />
          {isLastUserMessage && (
            <button
              onClick={() => setIsEditing(true)}
              className="absolute right-2 -bottom-2 opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded-full bg-gray-600/50 hover:bg-gray-600/80 text-white"
              title={t("chat.edit")}
            >
              <RiEdit2Line size={16} />
            </button>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="flex justify-end w-full">
      <div className="rounded-3xl bg-[#2f2f2f]/40 px-4 py-3 my-2 max-w-[80%]">
        <div className="flex flex-col">
          <textarea
            ref={textareaRef}
            value={editedContent}
            onChange={(e) => setEditedContent(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                handleSave();
              }
              if (e.key === "Escape") {
                setIsEditing(false);
                setEditedContent(message.message);
              }
            }}
            className="w-full bg-transparent text-white outline-none resize-none chat-input p-1 rounded"
            style={{
              minHeight: "20px",
            }}
          />
          <div className="flex justify-end mt-2">
            <button
              onClick={() => {
                setIsEditing(false);
                setEditedContent(message.message);
              }}
              className="text-gray-400 hover:text-gray-200 text-sm mr-3"
            >
              {t("cancel")}
            </button>
            <button
              onClick={handleSave}
              className="flex items-center gap-1 text-[#FB2671] hover:text-[#FB2671]/80 text-sm"
            >
              <FiSend size={14} />
              {t("send")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
