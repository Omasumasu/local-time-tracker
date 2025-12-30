import { api } from '../api';
import type {
  Task,
  TimeEntryWithRelations,
  Artifact,
  CreateTask,
  UpdateTask,
  UpdateEntry,
  CreateArtifact,
  ExportData,
  ImportResult,
  ListEntriesFilter,
  AppState,
} from '../types';

type Listener = () => void;

class Store {
  private state: AppState = {
    tasks: [],
    entries: [],
    runningEntry: null,
    artifacts: [],
    isLoading: false,
    error: null,
  };

  private listeners: Set<Listener> = new Set();
  private timerInterval: number | null = null;

  // State getters
  getState(): AppState {
    return this.state;
  }

  // Subscribe to state changes
  subscribe(listener: Listener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notify(): void {
    this.listeners.forEach((listener) => listener());
  }

  private setState(partial: Partial<AppState>): void {
    this.state = { ...this.state, ...partial };
    this.notify();
  }

  private setLoading(isLoading: boolean): void {
    this.setState({ isLoading });
  }

  private setError(error: string | null): void {
    this.setState({ error });
  }

  // Initialize store
  async initialize(): Promise<void> {
    this.setLoading(true);
    try {
      await Promise.all([
        this.loadTasks(),
        this.loadEntries(),
        this.loadRunningEntry(),
      ]);
      this.startTimerUpdate();
    } catch (err) {
      this.setError(err instanceof Error ? err.message : 'Initialization failed');
    } finally {
      this.setLoading(false);
    }
  }

  // Timer update for running entry
  private startTimerUpdate(): void {
    if (this.timerInterval) {
      clearInterval(this.timerInterval);
    }
    this.timerInterval = window.setInterval(() => {
      if (this.state.runningEntry) {
        this.notify();
      }
    }, 1000);
  }

  // Tasks
  async loadTasks(includeArchived: boolean = false): Promise<void> {
    const tasks = await api.tasks.list(includeArchived);
    this.setState({ tasks });
  }

  async createTask(task: CreateTask): Promise<Task> {
    const newTask = await api.tasks.create(task);
    this.setState({ tasks: [...this.state.tasks, newTask] });
    return newTask;
  }

  async updateTask(id: string, update: UpdateTask): Promise<Task> {
    const updatedTask = await api.tasks.update(id, update);
    this.setState({
      tasks: this.state.tasks.map((t) => (t.id === id ? updatedTask : t)),
    });
    return updatedTask;
  }

  async archiveTask(id: string, archived: boolean): Promise<void> {
    await api.tasks.archive(id, archived);
    this.setState({
      tasks: this.state.tasks.map((t) =>
        t.id === id ? { ...t, archived } : t
      ),
    });
  }

  // Entries
  async loadEntries(filter: ListEntriesFilter = {}): Promise<void> {
    const entries = await api.entries.list(filter);
    this.setState({ entries });
  }

  async loadRunningEntry(): Promise<void> {
    const runningEntry = await api.entries.getRunning();
    this.setState({ runningEntry });
  }

  async startEntry(taskId?: string, memo?: string): Promise<void> {
    await api.entries.start(taskId, memo);
    await this.loadRunningEntry();
    await this.loadEntries();
  }

  async stopEntry(memo?: string): Promise<void> {
    const { runningEntry } = this.state;
    if (!runningEntry) return;

    await api.entries.stop(runningEntry.id, memo);
    this.setState({ runningEntry: null });
    await this.loadEntries();
  }

  async updateEntry(id: string, update: UpdateEntry): Promise<void> {
    await api.entries.update(id, update);
    await this.loadEntries();
    if (this.state.runningEntry?.id === id) {
      await this.loadRunningEntry();
    }
  }

  async deleteEntry(id: string): Promise<void> {
    await api.entries.delete(id);
    if (this.state.runningEntry?.id === id) {
      this.setState({ runningEntry: null });
    }
    await this.loadEntries();
  }

  // Artifacts
  async loadArtifacts(limit?: number): Promise<void> {
    const artifacts = await api.artifacts.list(limit);
    this.setState({ artifacts });
  }

  async createArtifact(artifact: CreateArtifact, entryId?: string): Promise<Artifact> {
    const newArtifact = await api.artifacts.create(artifact, entryId);
    this.setState({ artifacts: [...this.state.artifacts, newArtifact] });
    if (entryId) {
      await this.loadEntries();
    }
    return newArtifact;
  }

  async linkArtifact(entryId: string, artifactId: string): Promise<void> {
    await api.artifacts.link(entryId, artifactId);
    await this.loadEntries();
  }

  async unlinkArtifact(entryId: string, artifactId: string): Promise<void> {
    await api.artifacts.unlink(entryId, artifactId);
    await this.loadEntries();
  }

  async deleteArtifact(id: string): Promise<void> {
    await api.artifacts.delete(id);
    this.setState({
      artifacts: this.state.artifacts.filter((a) => a.id !== id),
    });
    await this.loadEntries();
  }

  // Export/Import
  async exportData(): Promise<ExportData> {
    return api.export.exportData();
  }

  async importData(data: ExportData, merge: boolean): Promise<ImportResult> {
    const result = await api.export.importData(data, merge);
    await this.initialize();
    return result;
  }

  async exportParquet(outputDir: string): Promise<string[]> {
    return api.export.exportParquet(outputDir);
  }

  // Utility
  getTaskById(id: string): Task | undefined {
    return this.state.tasks.find((t) => t.id === id);
  }

  getActiveTasks(): Task[] {
    return this.state.tasks.filter((t) => !t.archived);
  }

  getEntriesGroupedByDate(): Map<string, TimeEntryWithRelations[]> {
    const groups = new Map<string, TimeEntryWithRelations[]>();

    for (const entry of this.state.entries) {
      const dateKey = entry.started_at.split('T')[0];
      const existing = groups.get(dateKey) || [];
      groups.set(dateKey, [...existing, entry]);
    }

    return groups;
  }

  // Cleanup
  destroy(): void {
    if (this.timerInterval) {
      clearInterval(this.timerInterval);
      this.timerInterval = null;
    }
    this.listeners.clear();
  }
}

// Singleton instance
export const store = new Store();
export default store;
