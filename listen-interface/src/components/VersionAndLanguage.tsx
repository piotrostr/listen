import { useTranslation } from "react-i18next";
import { useVersionStore } from "../store/versionStore";
import { LanguageSwitcher } from "./LanguageSwitcher";

export function VersionDisplay() {
  const { t } = useTranslation();
  const { version } = useVersionStore();
  return (
    <span className="text-xs text-gray-400">
      {t("layout.version")}: {version}
    </span>
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
