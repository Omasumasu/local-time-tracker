// Task types
export interface Task {
  id: string;
  name: string;
  description: string | null;
  color: string;
  archived: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateTask {
  name: string;
  description?: string;
  color?: string;
}

export interface UpdateTask {
  name?: string;
  description?: string;
  color?: string;
}

// TimeEntry types
export interface TimeEntry {
  id: string;
  task_id: string | null;
  started_at: string;
  ended_at: string | null;
  memo: string | null;
  created_at: string;
  updated_at: string;
}

export interface TimeEntryWithRelations extends TimeEntry {
  task: Task | null;
  artifacts: Artifact[];
  duration_seconds: number | null;
}

export interface StartEntry {
  task_id?: string;
  memo?: string;
}

export interface UpdateEntry {
  task_id?: string | null;
  started_at?: string;
  ended_at?: string | null;
  memo?: string | null;
}

// Artifact types
export interface Artifact {
  id: string;
  name: string;
  artifact_type: string;
  reference: string | null;
  metadata: Record<string, unknown> | null;
  created_at: string;
}

export interface CreateArtifact {
  name: string;
  artifact_type: string;
  reference?: string;
  metadata?: Record<string, unknown>;
}

// Export/Import types
export interface ExportTimeEntry {
  id: string;
  task_id: string | null;
  started_at: string;
  ended_at: string | null;
  duration_seconds: number | null;
  memo: string | null;
  created_at: string;
  updated_at: string;
}

export interface EntryArtifact {
  entry_id: string;
  artifact_id: string;
}

export interface ExportData {
  version: string;
  exported_at: string;
  tasks: Task[];
  artifacts: Artifact[];
  time_entries: ExportTimeEntry[];
  entry_artifacts: EntryArtifact[];
}

export interface ImportResult {
  tasks_imported: number;
  entries_imported: number;
  artifacts_imported: number;
}

// Query filters
export interface ListEntriesFilter {
  from?: string;
  to?: string;
  task_id?: string;
  limit?: number;
}

// App state types
export interface AppState {
  tasks: Task[];
  entries: TimeEntryWithRelations[];
  runningEntry: TimeEntryWithRelations | null;
  artifacts: Artifact[];
  isLoading: boolean;
  error: string | null;
}
