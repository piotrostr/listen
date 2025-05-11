import { useTranslation } from "react-i18next";
import { GradientOutlineButtonMoreRounded } from "./GradientOutlineButtonMoreRounded";
import { OutlineButton } from "./OutlineButton";

export function PipelineMenu({
  status,
  setStatus,
  sendPipelineForExecution,
  executeFromEoa,
}: {
  status: "pending" | "approved" | "rejected";
  setStatus: (status: "pending" | "approved" | "rejected") => void;
  sendPipelineForExecution?: () => void;
  executeFromEoa?: () => void;
}) {
  const { t } = useTranslation();

  const Container = ({ children }: { children: React.ReactNode }) => {
    return <div className="flex gap-2">{children}</div>;
  };

  const handleClickConfirm = () => {
    const handler = sendPipelineForExecution || executeFromEoa;
    console.log("handler", handler);
    if (handler) {
      handler();
    }
  };

  switch (status) {
    case "pending":
      return (
        <Container>
          <>
            <OutlineButton
              text={t("pipelines.reject")}
              onClick={() => setStatus("rejected")}
              className="w-full rounded-[16px]"
            />
            <GradientOutlineButtonMoreRounded
              onClick={handleClickConfirm}
              text={t("pipelines.approve")}
            />
          </>
        </Container>
      );
    case "approved":
      return (
        <Container>
          <div className="text-green-400 flex items-center gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              fill="currentColor"
              className="w-5 h-5"
            >
              <path
                fillRule="evenodd"
                d="M2.25 12c0-5.385 4.365-9.75 9.75-9.75s9.75 4.365 9.75 9.75-4.365 9.75-9.75 9.75S2.25 17.385 2.25 12zm13.36-1.814a.75.75 0 10-1.22-.872l-3.236 4.53L9.53 12.22a.75.75 0 00-1.06 1.06l2.25 2.25a.75.75 0 001.14-.094l3.75-5.25z"
                clipRule="evenodd"
              />
            </svg>
            <span>{t("pipelines.pipeline_scheduled_for_execution")}</span>
          </div>
        </Container>
      );
    case "rejected":
      return (
        <Container>
          <div className="text-red-400 flex items-center gap-2">
            <span>{t("pipelines.pipeline_rejected")}</span>
          </div>
        </Container>
      );
  }
}
