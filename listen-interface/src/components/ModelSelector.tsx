import claudeIcon from "../assets/icons/claude.png";
import geminiIcon from "../assets/icons/gemini.webp";
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
      disabled={id === "claude"} // tmp disable claude till get GCP credits
      className={`p-4 border-2 ${
        isSelected ? "border-[#2D2D2D]" : "border-transparent"
      } rounded-lg bg-black/40 backdrop-blur-sm hover:border-[#2D2D2D] transition-all`}
    >
      <div className="flex flex-row items-center justify-center gap-2">
        <img src={iconPath} alt={label} className="w-6 h-6" />
        <span className="text-sm">{label}</span>
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
