// Small UI state that many screens share. Data from the backend goes
// through TanStack Query instead.
import { create } from "zustand";

export type Screen = "today" | "tasks";

interface UiState {
  screen: Screen;
  setScreen: (s: Screen) => void;
  captureOpen: boolean;
  setCaptureOpen: (open: boolean) => void;
  agendaOpen: boolean;
  toggleAgenda: () => void;
  /** "Just one thing" mode on the Today view hides everything but the next step. */
  justOne: boolean;
  setJustOne: (on: boolean) => void;
}

export const useUi = create<UiState>((set) => ({
  screen: "today",
  setScreen: (screen) => set({ screen }),
  captureOpen: false,
  setCaptureOpen: (captureOpen) => set({ captureOpen }),
  agendaOpen: true,
  toggleAgenda: () => set((s) => ({ agendaOpen: !s.agendaOpen })),
  justOne: false,
  setJustOne: (justOne) => set({ justOne }),
}));
