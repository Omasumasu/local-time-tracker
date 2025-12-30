import { store } from '../store';
import type { Task } from '../types';

export class TaskManager {
  private element: HTMLElement;
  private showArchived: boolean = false;

  constructor(container: HTMLElement) {
    this.element = container;
    this.render();
    store.subscribe(() => this.render());
  }

  private render(): void {
    const state = store.getState();
    const tasks = this.showArchived
      ? state.tasks
      : state.tasks.filter((t) => !t.archived);

    this.element.innerHTML = `
      <div class="task-manager">
        <div class="task-header">
          <h3>TASKS</h3>
          <button class="btn btn-sm btn-primary" id="add-task-btn">+ \u8ffd\u52a0</button>
        </div>

        <div class="task-list" id="task-list">
          ${tasks.length === 0 ? '<p class="empty-hint">\u30bf\u30b9\u30af\u304c\u3042\u308a\u307e\u305b\u3093</p>' : ''}
          ${tasks.map((task) => this.renderTask(task)).join('')}
        </div>

        <div class="task-footer">
          <label class="checkbox-label">
            <input type="checkbox" id="show-archived" ${this.showArchived ? 'checked' : ''} />
            \u30a2\u30fc\u30ab\u30a4\u30d6\u3082\u8868\u793a
          </label>
        </div>
      </div>

      <div class="modal-overlay hidden" id="task-modal-overlay">
        <div class="modal-content modal-sm">
          <div class="modal-header">
            <h3 id="task-modal-title">\u30bf\u30b9\u30af\u3092\u8ffd\u52a0</h3>
            <button class="modal-close-btn" id="task-modal-close">&times;</button>
          </div>
          <div class="modal-body">
            <form id="task-form">
              <input type="hidden" id="task-id" name="id" />

              <div class="form-group">
                <label for="task-name">\u540d\u524d</label>
                <input type="text" id="task-name" name="name" required />
              </div>

              <div class="form-group">
                <label for="task-description">\u8aac\u660e</label>
                <textarea id="task-description" name="description" rows="2"></textarea>
              </div>

              <div class="form-group">
                <label for="task-color">\u30ab\u30e9\u30fc</label>
                <input type="color" id="task-color" name="color" value="#3b82f6" />
              </div>

              <div class="form-actions">
                <button type="button" class="btn btn-secondary" id="task-cancel-btn">
                  \u30ad\u30e3\u30f3\u30bb\u30eb
                </button>
                <button type="submit" class="btn btn-primary">
                  \u4fdd\u5b58
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    `;

    this.bindEvents();
  }

  private renderTask(task: Task): string {
    return `
      <div class="task-item ${task.archived ? 'archived' : ''}" data-task-id="${task.id}">
        <div class="task-color" style="background-color: ${task.color}"></div>
        <div class="task-info">
          <span class="task-name">${this.escapeHtml(task.name)}</span>
          ${task.description ? `<span class="task-desc">${this.escapeHtml(task.description)}</span>` : ''}
        </div>
        <div class="task-actions">
          <button class="btn-icon edit-task-btn" data-task-id="${task.id}" title="\u7de8\u96c6">
            \u270f
          </button>
          <button class="btn-icon archive-task-btn" data-task-id="${task.id}" data-archived="${task.archived}" title="${task.archived ? '\u5fa9\u5143' : '\u30a2\u30fc\u30ab\u30a4\u30d6'}">
            ${task.archived ? '\u21a9' : '\ud83d\udce6'}
          </button>
        </div>
      </div>
    `;
  }

  private bindEvents(): void {
    // Add task button
    this.element.querySelector('#add-task-btn')?.addEventListener('click', () => {
      this.openModal();
    });

    // Show archived checkbox
    this.element.querySelector('#show-archived')?.addEventListener('change', (e) => {
      this.showArchived = (e.target as HTMLInputElement).checked;
      store.loadTasks(this.showArchived);
    });

    // Edit task buttons
    this.element.querySelectorAll('.edit-task-btn').forEach((btn) => {
      btn.addEventListener('click', (e) => {
        const taskId = (e.currentTarget as HTMLElement).dataset.taskId;
        if (taskId) {
          const task = store.getTaskById(taskId);
          if (task) this.openModal(task);
        }
      });
    });

    // Archive task buttons
    this.element.querySelectorAll('.archive-task-btn').forEach((btn) => {
      btn.addEventListener('click', async (e) => {
        const target = e.currentTarget as HTMLElement;
        const taskId = target.dataset.taskId;
        const isArchived = target.dataset.archived === 'true';

        if (taskId) {
          await store.archiveTask(taskId, !isArchived);
        }
      });
    });

    // Modal events
    this.element.querySelector('#task-modal-overlay')?.addEventListener('click', (e) => {
      if (e.target === e.currentTarget) {
        this.closeModal();
      }
    });

    this.element.querySelector('#task-modal-close')?.addEventListener('click', () => {
      this.closeModal();
    });

    this.element.querySelector('#task-cancel-btn')?.addEventListener('click', () => {
      this.closeModal();
    });

    this.element.querySelector('#task-form')?.addEventListener('submit', (e) => {
      this.handleSubmit(e);
    });
  }

  private openModal(task?: Task): void {
    const modal = this.element.querySelector('#task-modal-overlay');
    const title = this.element.querySelector('#task-modal-title');
    const idInput = this.element.querySelector('#task-id') as HTMLInputElement;
    const nameInput = this.element.querySelector('#task-name') as HTMLInputElement;
    const descInput = this.element.querySelector('#task-description') as HTMLTextAreaElement;
    const colorInput = this.element.querySelector('#task-color') as HTMLInputElement;

    if (task) {
      title!.textContent = '\u30bf\u30b9\u30af\u3092\u7de8\u96c6';
      idInput.value = task.id;
      nameInput.value = task.name;
      descInput.value = task.description || '';
      colorInput.value = task.color;
    } else {
      title!.textContent = '\u30bf\u30b9\u30af\u3092\u8ffd\u52a0';
      idInput.value = '';
      nameInput.value = '';
      descInput.value = '';
      colorInput.value = '#3b82f6';
    }

    modal?.classList.remove('hidden');
    nameInput.focus();
  }

  private closeModal(): void {
    const modal = this.element.querySelector('#task-modal-overlay');
    modal?.classList.add('hidden');
  }

  private async handleSubmit(e: Event): Promise<void> {
    e.preventDefault();

    const form = e.target as HTMLFormElement;
    const formData = new FormData(form);

    const id = formData.get('id') as string;
    const name = formData.get('name') as string;
    const description = formData.get('description') as string;
    const color = formData.get('color') as string;

    try {
      if (id) {
        await store.updateTask(id, {
          name,
          description: description || undefined,
          color,
        });
      } else {
        await store.createTask({
          name,
          description: description || undefined,
          color,
        });
      }
      this.closeModal();
    } catch (err) {
      console.error('Failed to save task:', err);
      alert('\u4fdd\u5b58\u306b\u5931\u6557\u3057\u307e\u3057\u305f');
    }
  }

  private escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }
}
