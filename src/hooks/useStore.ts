import { useCallback, useSyncExternalStore } from 'react';
import { api } from '@/api';
import type {
  Task,
  TimeEntryWithRelations,
  Artifact,
  CreateTask,
  UpdateTask,
  UpdateEntry,
  CreateArtifact,
  ExportData,
  ListEntriesFilter,
} from '@/types';

interface AppState {
  tasks: Task[];
  entries: TimeEntryWithRelations[];
  runningEntry: TimeEntryWithRelations | null;
  artifacts: Artifact[];
  isLoading: boolean;
  error: string | null;
}

const initialState: AppState = {
  tasks: [],
  entries: [],
  runningEntry: null,
  artifacts: [],
  isLoading: false,
  error: null,
};

let state = initialState;
const listeners = new Set<() => void>();

function getState() {
  return state;
}

function setState(partial: Partial<AppState>) {
  state = { ...state, ...partial };
  listeners.forEach((listener) => listener());
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

// Timer interval
let timerInterval: number | null = null;

function startTimerUpdate() {
  if (timerInterval) {
    clearInterval(timerInterval);
  }
  timerInterval = window.setInterval(() => {
    if (state.runningEntry) {
      listeners.forEach((listener) => listener());
    }
  }, 1000);
}

export function useStore() {
  const currentState = useSyncExternalStore(subscribe, getState, getState);

  const initialize = useCallback(async () => {
    setState({ isLoading: true });
    try {
      const [tasks, entries, runningEntry] = await Promise.all([
        api.tasks.list(),
        api.entries.list(),
        api.entries.getRunning(),
      ]);
      setState({ tasks, entries, runningEntry, isLoading: false });
      startTimerUpdate();
    } catch (err) {
      setState({
        error: err instanceof Error ? err.message : 'Initialization failed',
        isLoading: false,
      });
    }
  }, []);

  // Tasks
  const loadTasks = useCallback(async (includeArchived = false) => {
    const tasks = await api.tasks.list(includeArchived);
    setState({ tasks });
  }, []);

  const createTask = useCallback(async (task: CreateTask) => {
    const newTask = await api.tasks.create(task);
    setState({ tasks: [...state.tasks, newTask] });
    return newTask;
  }, []);

  const updateTask = useCallback(async (id: string, update: UpdateTask) => {
    const updatedTask = await api.tasks.update(id, update);
    setState({
      tasks: state.tasks.map((t) => (t.id === id ? updatedTask : t)),
    });
    return updatedTask;
  }, []);

  const archiveTask = useCallback(async (id: string, archived: boolean) => {
    await api.tasks.archive(id, archived);
    setState({
      tasks: state.tasks.map((t) => (t.id === id ? { ...t, archived } : t)),
    });
  }, []);

  // Entries
  const loadEntries = useCallback(async (filter: ListEntriesFilter = {}) => {
    const entries = await api.entries.list(filter);
    setState({ entries });
  }, []);

  const loadRunningEntry = useCallback(async () => {
    const runningEntry = await api.entries.getRunning();
    setState({ runningEntry });
  }, []);

  const startEntry = useCallback(async (taskId?: string, memo?: string) => {
    await api.entries.start(taskId, memo);
    await loadRunningEntry();
    await loadEntries();
  }, [loadRunningEntry, loadEntries]);

  const stopEntry = useCallback(async (memo?: string) => {
    if (!state.runningEntry) return;
    await api.entries.stop(state.runningEntry.id, memo);
    setState({ runningEntry: null });
    await loadEntries();
  }, [loadEntries]);

  const updateEntry = useCallback(async (id: string, update: UpdateEntry) => {
    await api.entries.update(id, update);
    await loadEntries();
    if (state.runningEntry?.id === id) {
      await loadRunningEntry();
    }
  }, [loadEntries, loadRunningEntry]);

  const deleteEntry = useCallback(async (id: string) => {
    await api.entries.delete(id);
    if (state.runningEntry?.id === id) {
      setState({ runningEntry: null });
    }
    await loadEntries();
  }, [loadEntries]);

  // Artifacts
  const loadArtifacts = useCallback(async (limit?: number) => {
    const artifacts = await api.artifacts.list(limit);
    setState({ artifacts });
  }, []);

  const createArtifact = useCallback(async (artifact: CreateArtifact, entryId?: string) => {
    const newArtifact = await api.artifacts.create(artifact, entryId);
    setState({ artifacts: [...state.artifacts, newArtifact] });
    if (entryId) {
      await loadEntries();
    }
    return newArtifact;
  }, [loadEntries]);

  const linkArtifact = useCallback(async (entryId: string, artifactId: string) => {
    await api.artifacts.link(entryId, artifactId);
    await loadEntries();
  }, [loadEntries]);

  const unlinkArtifact = useCallback(async (entryId: string, artifactId: string) => {
    await api.artifacts.unlink(entryId, artifactId);
    await loadEntries();
  }, [loadEntries]);

  const deleteArtifact = useCallback(async (id: string) => {
    await api.artifacts.delete(id);
    setState({
      artifacts: state.artifacts.filter((a) => a.id !== id),
    });
    await loadEntries();
  }, [loadEntries]);

  // Export/Import
  const exportData = useCallback(async () => {
    return api.export.exportData();
  }, []);

  const importData = useCallback(async (data: ExportData, merge: boolean) => {
    const result = await api.export.importData(data, merge);
    await initialize();
    return result;
  }, [initialize]);

  const exportParquet = useCallback(async (outputDir: string) => {
    return api.export.exportParquet(outputDir);
  }, []);

  // Reports
  const getMonthlyReport = useCallback(async (year: number, month: number) => {
    return api.reports.getMonthlyReport(year, month);
  }, []);

  const getAvailableMonths = useCallback(async () => {
    return api.reports.getAvailableMonths();
  }, []);

  // Utilities
  const getTaskById = useCallback((id: string) => {
    return state.tasks.find((t) => t.id === id);
  }, []);

  const getActiveTasks = useCallback(() => {
    return state.tasks.filter((t) => !t.archived);
  }, []);

  return {
    ...currentState,
    initialize,
    loadTasks,
    createTask,
    updateTask,
    archiveTask,
    loadEntries,
    loadRunningEntry,
    startEntry,
    stopEntry,
    updateEntry,
    deleteEntry,
    loadArtifacts,
    createArtifact,
    linkArtifact,
    unlinkArtifact,
    deleteArtifact,
    exportData,
    importData,
    exportParquet,
    getMonthlyReport,
    getAvailableMonths,
    getTaskById,
    getActiveTasks,
  };
}
