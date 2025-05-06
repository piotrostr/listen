import { useTranslation } from "react-i18next";
import { AddToHomeScreenGuide } from "./AddToHomeScreenGuide";
import { AddToHomeScreenIcon } from "./AddToHomeScreenIcon";
import { GradientOutlineButton } from "./GradientOutlineButton";
import { OutlineButton } from "./OutlineButton";
import { Rectangle } from "./Rectangle";

export const AddToHomeScreenPopup = ({
  handleClickOk,
  handleClickLater,
}: {
  handleClickOk: () => void;
  handleClickLater: () => void;
}) => {
  const { t } = useTranslation();

  return (
    <div className="fixed inset-0 z-50 flex items-end backdrop-blur-sm bg-black/50">
      <div className="relative w-full max-w-md bg-[#151518] border border-[#2D2D2D] rounded-t-[24px] shadow-xl pb-9">
        <div className="flex justify-center pt-2">
          <Rectangle />
        </div>
        <div className="p-6">
          <div className="flex flex-col items-center justify-center gap-3">
            <AddToHomeScreenIcon />
            <div className="font-space-grotesk text-2xl leading-8 tracking-[-0.03em] text-left align-middle font-normal text-white">
              {t("add_to_home_screen.title")}
            </div>
            <div className="font-space-grotesk text-sm leading-6 tracking-[0%] text-left align-middle font-bold text-[#989898]">
              {t("add_to_home_screen.description")}
            </div>
            <div className="font-space-grotesk text-sm leading-6 tracking-[0%] text-left align-middle font-normal text-[#737373]">
              {t("add_to_home_screen.extended_description")}
            </div>
            <AddToHomeScreenGuide />
            <GradientOutlineButton
              text={t("add_to_home_screen.added")}
              onClick={handleClickOk}
            />
            <OutlineButton
              text={t("add_to_home_screen.later")}
              onClick={handleClickLater}
            />
          </div>
        </div>
      </div>
    </div>
  );
};
