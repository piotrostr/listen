import { z } from "zod";

const LevelSchema = z.object({
  n: z.number(),
  px: z.string(),
  sz: z.string(),
});

export const L2OrderbookSnapshotSchema = z.object({
  coin: z.string(),
  levels: z.array(z.array(LevelSchema)),
  time: z.number(),
});

export type L2OrderbookSnapshot = z.infer<typeof L2OrderbookSnapshotSchema>;
