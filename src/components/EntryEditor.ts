import { store } from '../store';
import { toDatetimeLocal, fromDatetimeLocal } from '../utils/format';
import type { TimeEntryWithRelations, Task } from '../types';

export class EntryEditor {
  private element: HTMLElement;
  private currentEntry: TimeEntryWithRelations | null = null;

  constructor(container: HTMLElement) {
    this.element = container;
    this.render();
  }

  open(entry: TimeEntryWithRelations): void {
    this.currentEntry = entry;
    this.renderForm();
    this.element.classList.add('open');
  }

  close(): void {
    this.currentEntry = null;
    this.element.classList.remove('open');
  }

  private render(): void {
    this.element.innerHTML = `
      <div class="modal-overlay" id="entry-editor-overlay">
        <div class="modal-content">
          <div class="modal-header">
            <h3>\u8a18\u9332\u306e\u7de8\u96c6</h3>
            <button class="modal-close-btn" id="entry-editor-close">&times;</button>
          </div>
          <div class="modal-body" id="entry-editor-form">
          </div>
        </div>
      </div>
    `;

    this.element.querySelector('#entry-editor-overlay')?.addEventListener('click', (e) => {
      if (e.target === e.currentTarget) {
        this.close();
      }
    });

    this.element.querySelector('#entry-editor-close')?.addEventListener('click', () => {
      this.close();
    });
  }

  private renderForm(): void {
    const formContainer = this.element.querySelector('#entry-editor-form');
    if (!formContainer || !this.currentEntry) return;

    const entry = this.currentEntry;
    const tasks = store.getActiveTasks();

    formContainer.innerHTML = `
      <form id="entry-form">
        <div class="form-group">
          <label for="edit-task">\u30bf\u30b9\u30af</label>
          <select id="edit-task" name="task_id">
            <option value="">-- \u672a\u5206\u985e --</option>
            ${tasks.map((t: Task) => `
              <option value="${t.id}" ${entry.task_id === t.id ? 'selected' : ''}>
                ${this.escapeHtml(t.name)}
              </option>
            `).join('')}
          </select>
        </div>

        <div class="form-group">
          <label for="edit-started-at">\u958b\u59cb\u6642\u523b</label>
          <input
            type="datetime-local"
            id="edit-started-at"
            name="started_at"
            value="${toDatetimeLocal(entry.started_at)}"
          />
        </div>

        <div class="form-group">
          <label for="edit-ended-at">\u7d42\u4e86\u6642\u523b</label>
          <input
            type="datetime-local"
            id="edit-ended-at"
            name="ended_at"
            value="${entry.ended_at ? toDatetimeLocal(entry.ended_at) : ''}"
            ${entry.ended_at === null ? 'disabled' : ''}
          />
          ${entry.ended_at === null ? '<span class="hint">\u8a08\u6e2c\u4e2d\u306e\u305f\u3081\u7de8\u96c6\u3067\u304d\u307e\u305b\u3093</span>' : ''}
        </div>

        <div class="form-group">
          <label for="edit-memo">\u30e1\u30e2</label>
          <textarea
            id="edit-memo"
            name="memo"
            rows="3"
            placeholder="\u30e1\u30e2\u3092\u5165\u529b..."
          >${entry.memo || ''}</textarea>
        </div>

        <div class="form-actions">
          <button type="button" class="btn btn-danger" id="delete-entry-btn">
            \u524a\u9664
          </button>
          <div class="form-actions-right">
            <button type="button" class="btn btn-secondary" id="cancel-edit-btn">
              \u30ad\u30e3\u30f3\u30bb\u30eb
            </button>
            <button type="submit" class="btn btn-primary">
              \u4fdd\u5b58
            </button>
          </div>
        </div>
      </form>
    `;

    // Bind form events
    const form = formContainer.querySelector('#entry-form');
    form?.addEventListener('submit', (e) => this.handleSubmit(e));

    formContainer.querySelector('#cancel-edit-btn')?.addEventListener('click', () => {
      this.close();
    });

    formContainer.querySelector('#delete-entry-btn')?.addEventListener('click', () => {
      this.handleDelete();
    });
  }

  private async handleSubmit(e: Event): Promise<void> {
    e.preventDefault();
    if (!this.currentEntry) return;

    const form = e.target as HTMLFormElement;
    const formData = new FormData(form);

    const taskId = formData.get('task_id') as string;
    const startedAt = formData.get('started_at') as string;
    const endedAt = formData.get('ended_at') as string;
    const memo = formData.get('memo') as string;

    try {
      await store.updateEntry(this.currentEntry.id, {
        task_id: taskId || null,
        started_at: fromDatetimeLocal(startedAt),
        ended_at: endedAt ? fromDatetimeLocal(endedAt) : null,
        memo: memo || null,
      });
      this.close();
    } catch (err) {
      console.error('Failed to update entry:', err);
      alert('\u66f4\u65b0\u306b\u5931\u6557\u3057\u307e\u3057\u305f');
    }
  }

  private async handleDelete(): Promise<void> {
    if (!this.currentEntry) return;

    if (!confirm('\u3053\u306e\u8a18\u9332\u3092\u524a\u9664\u3057\u307e\u3059\u304b\uff1f')) return;

    try {
      await store.deleteEntry(this.currentEntry.id);
      this.close();
    } catch (err) {
      console.error('Failed to delete entry:', err);
      alert('\u524a\u9664\u306b\u5931\u6557\u3057\u307e\u3057\u305f');
    }
  }

  private escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }
}
