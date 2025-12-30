import { store } from '../store';
import {
  formatDuration,
  formatTime,
  formatDate,
  calculateDuration,
} from '../utils/format';
import type { TimeEntryWithRelations } from '../types';

export type EditEntryCallback = (entry: TimeEntryWithRelations) => void;

export class Timeline {
  private element: HTMLElement;
  private onEditEntry: EditEntryCallback | null = null;

  constructor(container: HTMLElement) {
    this.element = container;
    this.render();
    store.subscribe(() => this.render());
  }

  setOnEditEntry(callback: EditEntryCallback): void {
    this.onEditEntry = callback;
  }

  private render(): void {
    const state = store.getState();
    const entriesGrouped = store.getEntriesGroupedByDate();

    // Sort date keys in descending order
    const sortedDates = Array.from(entriesGrouped.keys()).sort().reverse();

    if (sortedDates.length === 0) {
      this.element.innerHTML = `
        <div class="timeline-empty">
          <p>\u8a18\u9332\u304c\u3042\u308a\u307e\u305b\u3093</p>
          <p class="hint">\u30bf\u30a4\u30de\u30fc\u3092\u958b\u59cb\u3057\u3066\u4f5c\u696d\u6642\u9593\u3092\u8a18\u9332\u3057\u307e\u3057\u3087\u3046</p>
        </div>
      `;
      return;
    }

    let html = '<div class="timeline">';

    for (const dateKey of sortedDates) {
      const entries = entriesGrouped.get(dateKey) || [];
      const totalSeconds = entries.reduce((sum, entry) => {
        return sum + calculateDuration(entry.started_at, entry.ended_at);
      }, 0);

      html += `
        <div class="timeline-date-group">
          <div class="timeline-date-header">
            <span class="date">${formatDate(entries[0].started_at)}</span>
            <span class="total">\u5408\u8a08 ${formatDuration(totalSeconds)}</span>
          </div>
          <div class="timeline-entries">
      `;

      // Sort entries by started_at descending
      const sortedEntries = entries.sort(
        (a, b) => new Date(b.started_at).getTime() - new Date(a.started_at).getTime()
      );

      for (const entry of sortedEntries) {
        html += this.renderEntry(entry);
      }

      html += '</div></div>';
    }

    html += '</div>';
    this.element.innerHTML = html;

    // Bind edit button events
    this.element.querySelectorAll('.entry-edit-btn').forEach((btn) => {
      btn.addEventListener('click', (e) => {
        const entryId = (e.currentTarget as HTMLElement).dataset.entryId;
        if (entryId) {
          const entry = state.entries.find((e) => e.id === entryId);
          if (entry && this.onEditEntry) {
            this.onEditEntry(entry);
          }
        }
      });
    });
  }

  private renderEntry(entry: TimeEntryWithRelations): string {
    const isRunning = entry.ended_at === null;
    const duration = calculateDuration(entry.started_at, entry.ended_at);
    const durationStr = formatDuration(duration);
    const startTime = formatTime(entry.started_at);
    const endTime = entry.ended_at ? formatTime(entry.ended_at) : '';

    const taskName = entry.task?.name || '\u672a\u5206\u985e';
    const taskColor = entry.task?.color || '#6b7280';

    return `
      <div class="timeline-entry ${isRunning ? 'running' : ''}" data-entry-id="${entry.id}">
        <div class="entry-time">
          <span class="time-range">
            ${startTime}${endTime ? ' - ' + endTime : ''}
            ${isRunning ? '<span class="running-indicator">\u25cf \u8a08\u6e2c\u4e2d</span>' : ''}
          </span>
          <span class="duration">${durationStr}</span>
        </div>
        <div class="entry-content">
          <div class="entry-task" style="border-left-color: ${taskColor}">
            ${taskName}
          </div>
          ${entry.memo ? `<div class="entry-memo">${this.escapeHtml(entry.memo)}</div>` : ''}
          ${entry.artifacts.length > 0 ? this.renderArtifacts(entry.artifacts) : ''}
        </div>
        <button class="entry-edit-btn" data-entry-id="${entry.id}" title="\u7de8\u96c6">
          \u270f
        </button>
      </div>
    `;
  }

  private renderArtifacts(artifacts: { name: string; artifact_type: string }[]): string {
    return `
      <div class="entry-artifacts">
        ${artifacts.map((a) => `
          <span class="artifact-tag" title="${this.escapeHtml(a.artifact_type)}">
            \ud83d\udcce ${this.escapeHtml(a.name)}
          </span>
        `).join('')}
      </div>
    `;
  }

  private escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }
}
