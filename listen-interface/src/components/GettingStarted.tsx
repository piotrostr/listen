import { usePrivy } from "@privy-io/react-auth";
import { useTranslation } from "react-i18next";
import { FaExclamationTriangle } from "react-icons/fa";
import { PriceUpdates } from "./PriceUpdates";

export function GettingStarted() {
  const { login, ready } = usePrivy();
  const { t } = useTranslation();

  return (
    <div className="flex flex-col items-center gap-4 p-2">
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
          className="p-2 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm hover:bg-purple-500/10"
        >
          {t("getting_started.get_started")}
        </button>
        <p className="text-sm max-w-md text-center mt-3">
          {t("getting_started.questions")}
        </p>
      </div>

      <span className="text-sm my-8 mb-1 flex items-center gap-2">
        <FaExclamationTriangle className="text-yellow-500" />
        {t("getting_started.warning")}
      </span>

      {/* Divider */}
      <div className="border-t border-purple-500/30 w-full max-w-4xl" />

      {/* Screener Preview Section */}
      <div className="w-full max-w-4xl mx-auto">
        <div>
          <PriceUpdates />
        </div>
      </div>
    </div>
  );
}
