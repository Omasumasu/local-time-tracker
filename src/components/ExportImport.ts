import { save, open } from '@tauri-apps/plugin-dialog';
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs';
import { store } from '../store';
import type { ExportData } from '../types';

export class ExportImport {
  private element: HTMLElement;

  constructor(container: HTMLElement) {
    this.element = container;
    this.render();
  }

  private render(): void {
    this.element.innerHTML = `
      <div class="export-import">
        <h3>DATA</h3>
        <div class="action-list">
          <button class="btn btn-block" id="export-json-btn">
            \ud83d\udce4 JSON\u30a8\u30af\u30b9\u30dd\u30fc\u30c8
          </button>
          <button class="btn btn-block" id="import-json-btn">
            \ud83d\udce5 JSON\u30a4\u30f3\u30dd\u30fc\u30c8
          </button>
          <button class="btn btn-block" id="export-parquet-btn">
            \ud83d\udcca Parquet\u30a8\u30af\u30b9\u30dd\u30fc\u30c8
          </button>
        </div>
      </div>
    `;

    this.bindEvents();
  }

  private bindEvents(): void {
    this.element.querySelector('#export-json-btn')?.addEventListener('click', () => {
      this.handleExportJson();
    });

    this.element.querySelector('#import-json-btn')?.addEventListener('click', () => {
      this.handleImportJson();
    });

    this.element.querySelector('#export-parquet-btn')?.addEventListener('click', () => {
      this.handleExportParquet();
    });
  }

  private async handleExportJson(): Promise<void> {
    try {
      const data = await store.exportData();

      const filePath = await save({
        title: 'JSON\u30a8\u30af\u30b9\u30dd\u30fc\u30c8',
        defaultPath: `time-tracker-export-${this.getDateString()}.json`,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      });

      if (filePath) {
        await writeTextFile(filePath, JSON.stringify(data, null, 2));
        alert('\u30a8\u30af\u30b9\u30dd\u30fc\u30c8\u304c\u5b8c\u4e86\u3057\u307e\u3057\u305f');
      }
    } catch (err) {
      console.error('Export failed:', err);
      alert('\u30a8\u30af\u30b9\u30dd\u30fc\u30c8\u306b\u5931\u6557\u3057\u307e\u3057\u305f');
    }
  }

  private async handleImportJson(): Promise<void> {
    try {
      const filePath = await open({
        title: 'JSON\u30a4\u30f3\u30dd\u30fc\u30c8',
        filters: [{ name: 'JSON', extensions: ['json'] }],
        multiple: false,
      });

      if (!filePath || typeof filePath !== 'string') return;

      const content = await readTextFile(filePath);
      const data: ExportData = JSON.parse(content);

      // Validate data
      if (!data.version || !data.tasks || !data.time_entries) {
        throw new Error('\u7121\u52b9\u306a\u30d5\u30a1\u30a4\u30eb\u5f62\u5f0f\u3067\u3059');
      }

      const merge = confirm(
        '\u65e2\u5b58\u306e\u30c7\u30fc\u30bf\u3068\u30de\u30fc\u30b8\u3057\u307e\u3059\u304b\uff1f\n\n' +
        'OK: \u30de\u30fc\u30b8\uff08\u65e2\u5b58\u30c7\u30fc\u30bf\u3092\u4fdd\u6301\uff09\n' +
        '\u30ad\u30e3\u30f3\u30bb\u30eb: \u7f6e\u63db\uff08\u65e2\u5b58\u30c7\u30fc\u30bf\u3092\u524a\u9664\uff09'
      );

      const result = await store.importData(data, merge);

      alert(
        `\u30a4\u30f3\u30dd\u30fc\u30c8\u304c\u5b8c\u4e86\u3057\u307e\u3057\u305f\n\n` +
        `\u30bf\u30b9\u30af: ${result.tasks_imported}\u4ef6\n` +
        `\u8a18\u9332: ${result.entries_imported}\u4ef6\n` +
        `\u6210\u679c\u7269: ${result.artifacts_imported}\u4ef6`
      );
    } catch (err) {
      console.error('Import failed:', err);
      alert('\u30a4\u30f3\u30dd\u30fc\u30c8\u306b\u5931\u6557\u3057\u307e\u3057\u305f');
    }
  }

  private async handleExportParquet(): Promise<void> {
    try {
      const dirPath = await open({
        title: 'Parquet\u30a8\u30af\u30b9\u30dd\u30fc\u30c8\u5148\u30d5\u30a9\u30eb\u30c0',
        directory: true,
      });

      if (!dirPath || typeof dirPath !== 'string') return;

      const files = await store.exportParquet(dirPath);

      alert(
        `Parquet\u30a8\u30af\u30b9\u30dd\u30fc\u30c8\u304c\u5b8c\u4e86\u3057\u307e\u3057\u305f\n\n` +
        `\u51fa\u529b\u30d5\u30a1\u30a4\u30eb:\n${files.map((f) => `- ${f}`).join('\n')}`
      );
    } catch (err) {
      console.error('Parquet export failed:', err);
      alert('Parquet\u30a8\u30af\u30b9\u30dd\u30fc\u30c8\u306b\u5931\u6557\u3057\u307e\u3057\u305f');
    }
  }

  private getDateString(): string {
    const now = new Date();
    return now.toISOString().split('T')[0];
  }
}
