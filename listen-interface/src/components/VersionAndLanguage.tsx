import { useTranslation } from "react-i18next";
import { LanguageSwitcher } from "./LanguageSwitcher";

// Version Display Component
export function VersionAndLanguageDisplay() {
  const { t } = useTranslation();
  return (
    <div className="flex justify-around items-center w-full">
      <span className="text-xs text-gray-400">
        {t("layout.version")}: 2.2.2
      </span>
      <LanguageSwitcher />
    </div>
  );
}
