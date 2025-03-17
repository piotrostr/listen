import { useTranslation } from "react-i18next";
import { FaExclamationTriangle } from "react-icons/fa";

export function BetaWarning() {
  const { t } = useTranslation();

  return (
    <span className="text-xs mb-1 flex items-center gap-2 flex-col lg:flex-row text-center">
      <FaExclamationTriangle className="text-yellow-500" />
      {t("getting_started.warning")}
    </span>
  );
}
