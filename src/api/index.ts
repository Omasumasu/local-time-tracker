import { invoke } from '@tauri-apps/api/core';
import type {
  Task,
  CreateTask,
  UpdateTask,
  TimeEntry,
  TimeEntryWithRelations,
  UpdateEntry,
  Artifact,
  CreateArtifact,
  ExportData,
  ImportResult,
  ListEntriesFilter,
  MonthlyReport,
} from '../types';

// Tasks API
export const tasksApi = {
  list: (includeArchived: boolean = false): Promise<Task[]> => {
    return invoke('list_tasks', { includeArchived });
  },

  create: (task: CreateTask): Promise<Task> => {
    return invoke('create_task', { task });
  },

  update: (id: string, update: UpdateTask): Promise<Task> => {
    return invoke('update_task', { id, update });
  },

  archive: (id: string, archived: boolean): Promise<void> => {
    return invoke('archive_task', { id, archived });
  },
};

// Entries API
export const entriesApi = {
  list: (filter: ListEntriesFilter = {}): Promise<TimeEntryWithRelations[]> => {
    return invoke('list_entries', {
      from: filter.from,
      to: filter.to,
      taskId: filter.task_id,
      limit: filter.limit,
    });
  },

  getRunning: (): Promise<TimeEntryWithRelations | null> => {
    return invoke('get_running_entry');
  },

  start: (taskId?: string, memo?: string): Promise<TimeEntry> => {
    return invoke('start_entry', { taskId, memo });
  },

  stop: (id: string, memo?: string): Promise<TimeEntry> => {
    return invoke('stop_entry', { id, memo });
  },

  update: (id: string, update: UpdateEntry): Promise<TimeEntry> => {
    return invoke('update_entry', {
      id,
      update: {
        taskId: update.task_id,
        startedAt: update.started_at,
        endedAt: update.ended_at,
        memo: update.memo,
      },
    });
  },

  delete: (id: string): Promise<void> => {
    return invoke('delete_entry', { id });
  },
};

// Artifacts API
export const artifactsApi = {
  list: (limit?: number): Promise<Artifact[]> => {
    return invoke('list_artifacts', { limit });
  },

  create: (artifact: CreateArtifact, entryId?: string): Promise<Artifact> => {
    return invoke('create_artifact', { artifact, entryId });
  },

  link: (entryId: string, artifactId: string): Promise<void> => {
    return invoke('link_artifact', { entryId, artifactId });
  },

  unlink: (entryId: string, artifactId: string): Promise<void> => {
    return invoke('unlink_artifact', { entryId, artifactId });
  },

  delete: (id: string): Promise<void> => {
    return invoke('delete_artifact', { id });
  },
};

// Export/Import API
export const exportApi = {
  exportData: (): Promise<ExportData> => {
    return invoke('export_data');
  },

  importData: (data: ExportData, merge: boolean): Promise<ImportResult> => {
    return invoke('import_data', { data, merge });
  },

  exportParquet: (outputDir: string): Promise<string[]> => {
    return invoke('export_parquet', { outputDir });
  },
};

// Reports API
export const reportsApi = {
  getMonthlyReport: (year: number, month: number): Promise<MonthlyReport> => {
    return invoke('get_monthly_report', { year, month });
  },

  getAvailableMonths: (): Promise<[number, number][]> => {
    return invoke('get_available_months');
  },
};

// Aggregated API object
export const api = {
  tasks: tasksApi,
  entries: entriesApi,
  artifacts: artifactsApi,
  export: exportApi,
  reports: reportsApi,
};

export default api;
