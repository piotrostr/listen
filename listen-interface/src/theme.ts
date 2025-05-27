import { IThemeProvider, SciChartJsNavyTheme } from "scichart";

export interface AppTheme {
  SciChartJsTheme: IThemeProvider;
  VividGreen: string;
  VividRed: string;
}

export class MinimalAppTheme implements AppTheme {
  SciChartJsTheme = {
    ...new SciChartJsNavyTheme(),
    rolloverTooltipFill: "#1a1a1a",
    rolloverTooltipStroke: "#333333",
    rolloverTooltipTextColor: "#ffffff",
  };
  VividGreen = "#1FA67D";
  VividRed = "#ED7087";
}

export const appTheme = new MinimalAppTheme();
