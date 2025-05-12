import { useTranslation } from "react-i18next";

export const LanguageSwitcher = () => {
  const { i18n } = useTranslation();

  const toggleLanguage = () => {
    const newLang = i18n.language === "en" ? "zh" : "en";
    i18n.changeLanguage(newLang);
    localStorage.setItem("language", newLang);
  };

  const currentLanguage = i18n.language === "zh" ? "EN" : "中文";

  return (
    <button
      className="px-3 py-1 rounded-md border-2 border-[#212121] bg-transparent text-white hover:bg-[#212121] transition-colors"
      onClick={toggleLanguage}
    >
      {currentLanguage}
    </button>
  );
};

export default LanguageSwitcher;
