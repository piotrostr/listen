import { GradientOutlineButtonMoreRounded } from "./GradientOutlineButtonMoreRounded";
import { OutlineButton } from "./OutlineButton";

interface PercentageButtonProps {
  percentage: number;
  selectedPercentage: number;
  onClick: () => void;
}

export const PercentageButton = ({
  percentage,
  onClick,
  selectedPercentage,
}: PercentageButtonProps) => {
  const isSelected = selectedPercentage === percentage;
  if (isSelected) {
    return (
      <GradientOutlineButtonMoreRounded
        onClick={onClick}
        text={`${percentage}%`}
      />
    );
  }
  return <OutlineButton onClick={onClick} text={`${percentage}%`} />;
};

export const percentages = [
  { value: 25, multiplier: 0.25 },
  { value: 50, multiplier: 0.5 },
  { value: 75, multiplier: 0.75 },
  { value: 100, multiplier: 1 },
];
