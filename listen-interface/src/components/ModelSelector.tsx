import { ModelType } from "../store/settingsStore";

interface ModelOptionProps {
  id: ModelType;
  label: string;
  isSelected: boolean;
  onClick: (id: ModelType) => void;
}

function ModelOption({ id, label, isSelected, onClick }: ModelOptionProps) {
  return (
    <button
      onClick={() => onClick(id)}
      className={`p-4 border-2 ${
        isSelected ? "border-[#2D2D2D]" : "border-transparent"
      } rounded-lg bg-black/40 backdrop-blur-sm hover:border-[#2D2D2D] transition-all`}
    >
      <div className="flex flex-row justify-center">
        <span className="text-sm">{label}</span>
      </div>
    </button>
  );
}

const MODEL_OPTIONS = [
  {
    id: "claude" as const,
    label: "Claude",
  },
  {
    id: "gemini" as const,
    label: "Gemini",
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
          isSelected={selectedModel === option.id}
          onClick={onSelectModel}
        />
      ))}
    </div>
  );
}
