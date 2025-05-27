import { IThemeProvider, SciChartJsNavyTheme } from "scichart";

export interface AppTheme {
  SciChartJsTheme: IThemeProvider;
  VividGreen: string;
  VividRed: string;
}

export class MinimalAppTheme implements AppTheme {
  SciChartJsTheme = new SciChartJsNavyTheme();
  VividGreen = "#00ff00";
  VividRed = "#ff0000";
}

export const appTheme = new MinimalAppTheme();
