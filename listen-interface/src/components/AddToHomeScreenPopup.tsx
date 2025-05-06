import { useTranslation } from "react-i18next";
import { useHasAddedToHomeScreen } from "../hooks/useHasAddedToHomeScreen";
import { AddToHomeScreenGuide } from "./AddToHomeScreenGuide";
import { AddToHomeScreenIcon } from "./AddToHomeScreenIcon";
import { GradientOutlineButton } from "./GradientOutlineButton";
import { OutlineButton } from "./OutlineButton";

const Rectangle = () => (
  <svg
    width="40"
    height="5"
    viewBox="0 0 40 5"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect y="0.751221" width="40" height="4" rx="2" fill="#3E3B44" />
  </svg>
);

export const AddToHomeScreenPopup = () => {
  const { t } = useTranslation();
  const [_, updateHomeScreenStatus] = useHasAddedToHomeScreen();

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
      <div className="relative w-full max-w-md bg-[#151518]/90 border border-[#2D2D2D] rounded-xl shadow-xl">
        <div className="flex justify-center pt-2">
          <Rectangle />
        </div>
        <div className="p-6">
          <div className="flex flex-col items-center justify-center gap-4">
            <AddToHomeScreenIcon />
            <div className="font-space-grotesk text-2xl leading-8 tracking-[-0.03em] text-left align-middle font-normal text-white">
              {t("add_to_home_screen.title")}
            </div>
            <div className="font-space-grotesk text-sm leading-6 tracking-[0%] text-center align-middle font-bold text-[#989898]">
              {t("add_to_home_screen.description")}
            </div>
            <div className="font-space-grotesk text-sm leading-6 tracking-[0%] text-center align-middle font-normal text-[#737373]">
              {t("add_to_home_screen.extended_description")}
            </div>
            <AddToHomeScreenGuide />
            <GradientOutlineButton
              text={t("add_to_home_screen.added")}
              onClick={() => updateHomeScreenStatus(true)}
            />
            <OutlineButton
              text={t("add_to_home_screen.later")}
              onClick={() => updateHomeScreenStatus(false)}
            />
          </div>
        </div>
      </div>
    </div>
  );
};
