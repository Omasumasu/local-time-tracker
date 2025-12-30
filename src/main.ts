import { store } from './store';
import { Timer } from './components/Timer';
import { Timeline } from './components/Timeline';
import { EntryEditor } from './components/EntryEditor';
import { TaskManager } from './components/TaskManager';
import { ExportImport } from './components/ExportImport';
import './style.css';

async function main() {
  // Initialize store
  try {
    await store.initialize();
  } catch (err) {
    console.error('Failed to initialize:', err);
  }

  // Mount components
  const timerContainer = document.getElementById('timer');
  const timelineContainer = document.getElementById('timeline');
  const entryEditorContainer = document.getElementById('entry-editor');
  const taskManagerContainer = document.getElementById('task-manager');
  const exportImportContainer = document.getElementById('export-import');

  if (timerContainer) {
    new Timer(timerContainer);
  }

  let entryEditor: EntryEditor | null = null;
  if (entryEditorContainer) {
    entryEditor = new EntryEditor(entryEditorContainer);
  }

  if (timelineContainer) {
    const timeline = new Timeline(timelineContainer);
    if (entryEditor) {
      timeline.setOnEditEntry((entry) => {
        entryEditor!.open(entry);
      });
    }
  }

  if (taskManagerContainer) {
    new TaskManager(taskManagerContainer);
  }

  if (exportImportContainer) {
    new ExportImport(exportImportContainer);
  }
}

// Wait for DOM to be ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', main);
} else {
  main();
}
