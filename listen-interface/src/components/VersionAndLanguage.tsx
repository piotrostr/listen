import { useTranslation } from "react-i18next";
import { LanguageSwitcher } from "./LanguageSwitcher";

export function VersionDisplay() {
  const { t } = useTranslation();
  return (
    <span className="text-xs text-gray-400">{t("layout.version")}: 2.2.8</span>
  );
}

export function VersionAndLanguageDisplay() {
  return (
    <div className="flex justify-around items-center w-full">
      <VersionDisplay />
      <LanguageSwitcher />
    </div>
  );
}
