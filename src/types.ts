export type Side = 'w' | 'b'
export type Board = string[][]
export interface Position { board: Board; side: Side }
export interface EngineSettings { depth: number; time: number; threads: number; hash: number; cloud: boolean; cloudTimeout: number }
export interface Analysis { fen: string; bestmove: string; pv: string[]; score: number; mate: number | null; depth: number; time: number; source: string; warning?: string }
export interface WindowInfo { id: number; title: string; app: string }
export interface Resources { engine: boolean; model: boolean; runtime: boolean; desktop: boolean; resourceDir: string }
export const defaults: EngineSettings = { depth: 15, time: 1000, threads: 2, hash: 128, cloud: true, cloudTimeout: 1 }
