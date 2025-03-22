import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { IoCloudDownloadOutline } from "react-icons/io5";
import { useVersionStore } from "../store/versionStore";

export const VersionInitializer = () => {
  const { version, latestVersion, startPolling, stopPolling } =
    useVersionStore();
  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const { t } = useTranslation();

  // Start polling on component mount
  useEffect(() => {
    startPolling();
    return () => {
      stopPolling();
    };
  }, [startPolling, stopPolling]);

  // Check if version updated and show modal
  useEffect(() => {
    if (version && latestVersion && version !== latestVersion) {
      setShowUpdateModal(true);
    }
  }, [version, latestVersion]);

  // Handle refresh
  const handleRefresh = () => {
    window.location.reload();
  };

  return (
    <>
      {showUpdateModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50 bg-black/60">
          <div className="bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
            <div className="flex items-center justify-center mb-4 text-blue-400">
              <IoCloudDownloadOutline size={48} />
            </div>
            <h3 className="text-xl font-semibold text-center mb-2">
              {t("version.update_available")}
            </h3>
            <p className="text-gray-300 text-center mb-4">
              {t("version.new_version_message", { version })}
            </p>
            <div className="flex justify-between gap-4">
              <button
                onClick={() => setShowUpdateModal(false)}
                className="flex-1 px-4 py-2 rounded border border-gray-600 text-gray-300 hover:bg-gray-700 transition-colors"
              >
                {t("version.later")}
              </button>
              <button
                onClick={handleRefresh}
                className="flex-1 px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-700 transition-colors"
              >
                {t("version.update_now")}
              </button>
            </div>
            <p className="text-xs text-gray-400 mt-4 text-center">
              {t("version.current_version")}: {version}
              <br />
              {t("version.new_version")}: {latestVersion}
            </p>
          </div>
        </div>
      )}
    </>
  );
};
