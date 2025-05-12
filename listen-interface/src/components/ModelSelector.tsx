import claudeIcon from "../assets/icons/claude.png";
import deepseekIcon from "../assets/icons/deepseek.png";
import geminiIcon from "../assets/icons/gemini.webp";
import openaiIcon from "../assets/icons/openai.png";
import { ModelType } from "../store/settingsStore";

interface ModelOptionProps {
  id: ModelType;
  iconPath: string;
  label: string;
  isSelected: boolean;
  onClick: (id: ModelType) => void;
}

function ModelOption({
  id,
  iconPath,
  label,
  isSelected,
  onClick,
}: ModelOptionProps) {
  return (
    <button
      onClick={() => onClick(id)}
      className={`p-4 border-2 ${
        isSelected ? "border-[#2D2D2D]" : "border-transparent"
      } rounded-lg bg-black/40 backdrop-blur-sm hover:border-[#2D2D2D] transition-all`}
    >
      <div className="flex flex-row items-center justify-center gap-2">
        <img src={iconPath} alt={label} className="h-6" />
      </div>
    </button>
  );
}

const MODEL_OPTIONS = [
  {
    id: "claude" as const,
    label: "Claude",
    iconPath: claudeIcon,
  },
  {
    id: "gemini" as const,
    label: "Gemini",
    iconPath: geminiIcon,
  },
  {
    id: "openai" as const,
    label: "OpenAI",
    iconPath: openaiIcon,
  },
  {
    id: "deepseek" as const,
    label: "DeepSeek",
    iconPath: deepseekIcon,
  },
] as const;

interface ModelSelectorProps {
  selectedModel: ModelType;
  onSelectModel: (model: ModelType) => void;
}

export function ModelSelector({
  selectedModel,
  onSelectModel,
}: ModelSelectorProps) {
  return (
    <div className="flex flex-row gap-2">
      {MODEL_OPTIONS.map((option) => (
        <ModelOption
          key={option.id}
          id={option.id}
          label={option.label}
          iconPath={option.iconPath}
          isSelected={selectedModel === option.id}
          onClick={onSelectModel}
        />
      ))}
    </div>
  );
}
