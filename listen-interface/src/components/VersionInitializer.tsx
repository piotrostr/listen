import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { FaInfoCircle } from "react-icons/fa";
import { useVersionStore } from "../store/versionStore";

export const VersionInitializer = () => {
  const { version, latestVersion, startPolling, stopPolling } =
    useVersionStore();
  const [showUpdateNotification, setShowUpdateNotification] = useState(false);
  const { t } = useTranslation();

  // Start polling on component mount
  useEffect(() => {
    startPolling();
    return () => {
      stopPolling();
    };
  }, [startPolling, stopPolling]);

  // Check if version updated and show notification
  useEffect(() => {
    if (version && latestVersion && version !== latestVersion) {
      setShowUpdateNotification(true);
    }
  }, [version, latestVersion]);

  // Handle refresh
  const handleRefresh = () => {
    window.location.reload();
  };

  return (
    <>
      {showUpdateNotification && (
        <div className="fixed top-4 left-1/2 transform -translate-x-1/2 z-50 bg-black border border-[#2d2d2d] px-6 py-3 rounded-lg shadow-lg flex items-center justify-between gap-4 transition-all duration-300 ease-in-out opacity-100">
          <div className="flex items-center gap-3">
            <FaInfoCircle className="text-white text-lg flex-shrink-0" />
            <div className="flex flex-col">
              <div className="font-medium">
                {t("version.client_out_of_date")}
              </div>
              <div className="text-sm opacity-80">
                {t("version.please_refresh")}
              </div>
            </div>
          </div>
          <button
            className="bg-[#FB2771]/50 hover:bg-[#FB2771]/60 text-white py-1.5 px-3 rounded-md text-sm transition-colors ml-2"
            onClick={handleRefresh}
          >
            {t("version.refresh_now")}
          </button>
        </div>
      )}
    </>
  );
};
