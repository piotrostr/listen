import { usePrivy } from "@privy-io/react-auth";
import { useTranslation } from "react-i18next";
import { BetaWarning } from "./BetaWarning";

export function OldGettingStarted() {
  const { login, ready } = usePrivy();
  const { t } = useTranslation();

  return (
    <div className="flex flex-col items-center gap-4 p-2 w-full overflow-hidden">
      {/* Getting Started Section */}
      <div className="w-full max-w-2xl mx-auto flex flex-col items-center text-center gap-2">
        <h2 className="text-xl lg:text-2xl font-bold mt-5 mb-2">
          {t("getting_started.how_it_works")}
        </h2>
        <p className="text-sm lg:text-base">
          {t("getting_started.how_it_works_description")}
        </p>
        <p className="text-sm lg:text-base">{t("getting_started.step_1")}</p>
        <p className="text-sm lg:text-base">{t("getting_started.step_2")}</p>
        <p className="text-sm lg:text-base">{t("getting_started.step_3")}</p>
        <br />
        <button
          onClick={login}
          disabled={!ready}
          className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm hover:bg-[#2D2D2D]"
        >
          {t("getting_started.get_started")}
        </button>
        <p className="text-sm max-w-md text-center mt-3">
          {t("getting_started.questions")}
        </p>
      </div>
      <BetaWarning />
    </div>
  );
}
