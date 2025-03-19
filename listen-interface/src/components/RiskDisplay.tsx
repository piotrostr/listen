import { useTranslation } from "react-i18next";
import { FaCheckCircle, FaExclamationTriangle } from "react-icons/fa";
import { z } from "zod";

export const RiskSchema = z.object({
  name: z.string(),
  value: z.string(),
  description: z.string(),
  level: z.string(),
});

export const RiskAnalysisSchema = z.object({
  tokenProgram: z.string(),
  tokenType: z.string(),
  risks: z.array(RiskSchema),
});

export type RiskAnalysis = z.infer<typeof RiskAnalysisSchema>;
export type Risk = z.infer<typeof RiskSchema>;

const RiskDisplay = ({ risk }: { risk: Risk }) => {
  if (["warn", "danger"].includes(risk.level)) {
    return (
      <div key={risk.name} className="relative group">
        <div className="flex items-center gap-2">
          <div
            className={
              risk.level === "danger" ? "text-red-500" : "text-orange-500"
            }
          >
            <FaExclamationTriangle />
          </div>
          <span className="relative group cursor-pointer">
            <strong>{risk.name}</strong>
            {risk.value && `: ${risk.value}`}

            {/* Tooltip - moved inside the span element */}
            <div className="absolute left-0 bottom-full mb-2 hidden group-hover:block bg-black p-2 rounded shadow-lg z-10 max-w-xs text-white border border-gray-700">
              {risk.description}
            </div>
          </span>
        </div>
      </div>
    );
  }
  return null;
};

export function RiskAnalysisDisplay({
  riskAnalysis,
}: {
  riskAnalysis: RiskAnalysis;
}) {
  const { t } = useTranslation();
  return (
    <div className="p-3 text-sm mb-2">
      {riskAnalysis.risks.map((risk) => (
        <RiskDisplay key={risk.name} risk={risk} />
      ))}
      {riskAnalysis.risks.length === 0 && (
        <div className="text-green-300 flex items-center gap-2">
          <FaCheckCircle /> {t("tool_messages.no_risks_found")}
        </div>
      )}
    </div>
  );
}
