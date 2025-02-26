import { createFileRoute } from "@tanstack/react-router";
import { Pipelines } from "../components/Pipelines";

export const Route = createFileRoute("/pipelines")({
  component: Pipelines,
});
