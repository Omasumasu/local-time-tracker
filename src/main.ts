import { store } from './store';
import { Timer } from './components/Timer';
import { Timeline } from './components/Timeline';
import { EntryEditor } from './components/EntryEditor';
import { TaskManager } from './components/TaskManager';
import { ExportImport } from './components/ExportImport';
import { Report } from './components/Report';
import './style.css';

async function main() {
  // Initialize store
  try {
    await store.initialize();
  } catch (err) {
    console.error('Failed to initialize:', err);
  }

  // Tab navigation
  const tabs = document.querySelectorAll('.nav-tab');
  const tabContents = document.querySelectorAll('.tab-content');

  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      const targetTab = tab.getAttribute('data-tab');

      // Update tab styles
      tabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');

      // Show/hide content
      tabContents.forEach(content => {
        if (content.id === `${targetTab}-view`) {
          content.classList.add('active');
        } else {
          content.classList.remove('active');
        }
      });
    });
  });

  // Mount components
  const timerContainer = document.getElementById('timer');
  const timelineContainer = document.getElementById('timeline');
  const entryEditorContainer = document.getElementById('entry-editor');
  const taskManagerContainer = document.getElementById('task-manager');
  const exportImportContainer = document.getElementById('export-import');
  const reportContainer = document.getElementById('report');

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

  if (reportContainer) {
    new Report(reportContainer);
  }
}

// Wait for DOM to be ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', main);
} else {
  main();
}
