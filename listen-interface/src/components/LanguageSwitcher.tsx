import { useState } from "react";
import { useTranslation } from "react-i18next";

export const LanguageSwitcher = () => {
  const { i18n } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);

  const changeLanguage = (lng: string) => {
    i18n.changeLanguage(lng);
    localStorage.setItem("language", lng);
    setIsOpen(false);
  };

  const currentLanguage = i18n.language === "zh" ? "中文" : "EN";

  return (
    <div className="relative">
      <button
        className={`flex items-center px-3 py-1 rounded-md bg-gray-800 text-white hover:bg-gray-700 transition-colors`}
        onClick={() => setIsOpen(!isOpen)}
      >
        {currentLanguage}
        <svg
          className={`ml-2 w-4 h-4 transition-transform ${isOpen ? "rotate-180" : ""}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>

      {isOpen && (
        <div className="absolute right-0 mt-2 w-36 bg-gray-800 rounded-md shadow-lg overflow-hidden z-50">
          <div className="py-1">
            <button
              className={`block w-full text-left px-4 py-2 text-sm ${
                i18n.language === "en"
                  ? "bg-gray-700 text-white"
                  : "text-gray-200 hover:bg-gray-700"
              }`}
              onClick={() => changeLanguage("en")}
            >
              English
            </button>
            <button
              className={`block w-full text-left px-4 py-2 text-sm ${
                i18n.language === "zh"
                  ? "bg-gray-700 text-white"
                  : "text-gray-200 hover:bg-gray-700"
              }`}
              onClick={() => changeLanguage("zh")}
            >
              中文 (Chinese)
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default LanguageSwitcher;
