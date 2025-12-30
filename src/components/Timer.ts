import { store } from '../store';
import { formatDuration, calculateDuration } from '../utils/format';
import type { Task } from '../types';

export class Timer {
  private element: HTMLElement;
  private timerDisplay: HTMLElement | null = null;
  private taskSelect: HTMLSelectElement | null = null;
  private memoInput: HTMLInputElement | null = null;
  private startButton: HTMLButtonElement | null = null;
  private stopButton: HTMLButtonElement | null = null;

  constructor(container: HTMLElement) {
    this.element = container;
    this.render();
    this.bindEvents();
    store.subscribe(() => this.update());
  }

  private render(): void {
    this.element.innerHTML = `
      <div class="timer-container">
        <div class="timer-display" id="timer-display">0:00:00</div>
        <div class="timer-controls">
          <select id="task-select" class="task-select">
            <option value="">-- \u30bf\u30b9\u30af\u3092\u9078\u629e --</option>
          </select>
          <input type="text" id="memo-input" class="memo-input" placeholder="\u30e1\u30e2..." />
          <button id="start-button" class="btn btn-primary">
            <span class="btn-icon">\u25b6</span> \u958b\u59cb
          </button>
          <button id="stop-button" class="btn btn-danger hidden">
            <span class="btn-icon">\u25a0</span> \u505c\u6b62
          </button>
        </div>
      </div>
    `;

    this.timerDisplay = this.element.querySelector('#timer-display');
    this.taskSelect = this.element.querySelector('#task-select');
    this.memoInput = this.element.querySelector('#memo-input');
    this.startButton = this.element.querySelector('#start-button');
    this.stopButton = this.element.querySelector('#stop-button');

    this.updateTaskOptions();
    this.update();
  }

  private bindEvents(): void {
    this.startButton?.addEventListener('click', () => this.handleStart());
    this.stopButton?.addEventListener('click', () => this.handleStop());
  }

  private async handleStart(): Promise<void> {
    const taskId = this.taskSelect?.value || undefined;
    const memo = this.memoInput?.value || undefined;

    try {
      await store.startEntry(taskId, memo);
      if (this.memoInput) this.memoInput.value = '';
    } catch (err) {
      console.error('Failed to start entry:', err);
    }
  }

  private async handleStop(): Promise<void> {
    const memo = this.memoInput?.value || undefined;
    try {
      await store.stopEntry(memo);
    } catch (err) {
      console.error('Failed to stop entry:', err);
    }
  }

  private updateTaskOptions(): void {
    if (!this.taskSelect) return;

    const tasks = store.getActiveTasks();
    const currentValue = this.taskSelect.value;

    // Clear existing options except the first one
    while (this.taskSelect.options.length > 1) {
      this.taskSelect.remove(1);
    }

    // Add task options
    tasks.forEach((task: Task) => {
      const option = document.createElement('option');
      option.value = task.id;
      option.textContent = task.name;
      option.style.borderLeft = `4px solid ${task.color}`;
      this.taskSelect?.appendChild(option);
    });

    // Restore selection if still valid
    if (tasks.some((t) => t.id === currentValue)) {
      this.taskSelect.value = currentValue;
    }
  }

  private update(): void {
    const state = store.getState();
    const { runningEntry } = state;

    // Update timer display
    if (this.timerDisplay) {
      if (runningEntry) {
        const seconds = calculateDuration(runningEntry.started_at, null);
        this.timerDisplay.textContent = formatDuration(seconds);
        this.timerDisplay.classList.add('running');
      } else {
        this.timerDisplay.textContent = '0:00:00';
        this.timerDisplay.classList.remove('running');
      }
    }

    // Update buttons
    if (runningEntry) {
      this.startButton?.classList.add('hidden');
      this.stopButton?.classList.remove('hidden');
      if (this.taskSelect) this.taskSelect.disabled = true;

      // Show current task
      if (runningEntry.task_id && this.taskSelect) {
        this.taskSelect.value = runningEntry.task_id;
      }
    } else {
      this.startButton?.classList.remove('hidden');
      this.stopButton?.classList.add('hidden');
      if (this.taskSelect) this.taskSelect.disabled = false;
    }

    // Update task options
    this.updateTaskOptions();
  }
}
