import { Chart, registerables } from 'chart.js';
import { api } from '../api';
import type { MonthlyReport } from '../types';
import { formatDurationShort } from '../utils/format';

Chart.register(...registerables);

export class Report {
  private element: HTMLElement;
  private report: MonthlyReport | null = null;
  private availableMonths: [number, number][] = [];
  private selectedYear: number;
  private selectedMonth: number;
  private taskChart: Chart | null = null;
  private dailyChart: Chart | null = null;

  constructor(container: HTMLElement) {
    this.element = container;
    const now = new Date();
    this.selectedYear = now.getFullYear();
    this.selectedMonth = now.getMonth() + 1;
    this.initialize();
  }

  private async initialize(): Promise<void> {
    await this.loadAvailableMonths();
    this.render();
    await this.loadReport();
  }

  private async loadAvailableMonths(): Promise<void> {
    try {
      this.availableMonths = await api.reports.getAvailableMonths();
      if (this.availableMonths.length > 0) {
        const [year, month] = this.availableMonths[0];
        this.selectedYear = year;
        this.selectedMonth = month;
      }
    } catch (err) {
      console.error('Failed to load available months:', err);
    }
  }

  private async loadReport(): Promise<void> {
    try {
      this.report = await api.reports.getMonthlyReport(this.selectedYear, this.selectedMonth);
      this.updateCharts();
      this.updateSummary();
    } catch (err) {
      console.error('Failed to load report:', err);
    }
  }

  private render(): void {
    this.element.innerHTML = `
      <div class="report-container">
        <div class="report-header">
          <h2>月次レポート</h2>
          <div class="report-controls">
            <select id="month-select" class="month-select">
              ${this.renderMonthOptions()}
            </select>
            <button id="export-png-btn" class="btn btn-secondary btn-sm">PNG出力</button>
          </div>
        </div>

        <div class="report-content" id="report-content">
          <div class="report-summary" id="report-summary">
            <div class="summary-card">
              <div class="summary-label">合計時間</div>
              <div class="summary-value" id="total-time">--</div>
            </div>
            <div class="summary-card">
              <div class="summary-label">稼働日数</div>
              <div class="summary-value" id="working-days">--</div>
            </div>
            <div class="summary-card">
              <div class="summary-label">平均時間/日</div>
              <div class="summary-value" id="avg-time">--</div>
            </div>
            <div class="summary-card">
              <div class="summary-label">エントリ数</div>
              <div class="summary-value" id="entry-count">--</div>
            </div>
          </div>

          <div class="report-charts">
            <div class="chart-container">
              <h3>タスク別内訳</h3>
              <div class="chart-wrapper">
                <canvas id="task-chart"></canvas>
              </div>
            </div>
            <div class="chart-container chart-container-wide">
              <h3>日別推移</h3>
              <div class="chart-wrapper chart-wrapper-bar">
                <canvas id="daily-chart"></canvas>
              </div>
            </div>
          </div>

          <div class="report-task-list" id="task-list">
            <h3>タスク別詳細</h3>
            <div class="task-detail-list"></div>
          </div>
        </div>
      </div>
    `;

    this.bindEvents();
    this.initCharts();
  }

  private renderMonthOptions(): string {
    if (this.availableMonths.length === 0) {
      return `<option value="${this.selectedYear}-${this.selectedMonth}">${this.selectedYear}年${this.selectedMonth}月</option>`;
    }

    return this.availableMonths
      .map(([year, month]) => {
        const value = `${year}-${month}`;
        const selected = year === this.selectedYear && month === this.selectedMonth ? 'selected' : '';
        return `<option value="${value}" ${selected}>${year}年${month}月</option>`;
      })
      .join('');
  }

  private bindEvents(): void {
    const monthSelect = this.element.querySelector('#month-select') as HTMLSelectElement;
    monthSelect?.addEventListener('change', async (e) => {
      const [year, month] = (e.target as HTMLSelectElement).value.split('-').map(Number);
      this.selectedYear = year;
      this.selectedMonth = month;
      await this.loadReport();
    });

    const exportBtn = this.element.querySelector('#export-png-btn');
    exportBtn?.addEventListener('click', () => this.exportToPng());
  }

  private initCharts(): void {
    const taskCtx = document.getElementById('task-chart') as HTMLCanvasElement;
    const dailyCtx = document.getElementById('daily-chart') as HTMLCanvasElement;

    if (taskCtx) {
      this.taskChart = new Chart(taskCtx, {
        type: 'doughnut',
        data: {
          labels: [],
          datasets: [{
            data: [],
            backgroundColor: [],
            borderWidth: 0,
          }],
        },
        options: {
          responsive: true,
          maintainAspectRatio: true,
          plugins: {
            legend: {
              position: 'right',
              labels: {
                color: '#f8fafc',
                font: { size: 12 },
                padding: 12,
              },
            },
            tooltip: {
              callbacks: {
                label: (context) => {
                  const seconds = context.raw as number;
                  return ` ${formatDurationShort(seconds)}`;
                },
              },
            },
          },
        },
      });
    }

    if (dailyCtx) {
      this.dailyChart = new Chart(dailyCtx, {
        type: 'bar',
        data: {
          labels: [],
          datasets: [{
            label: '稼働時間',
            data: [],
            backgroundColor: '#3b82f6',
            borderRadius: 4,
          }],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          scales: {
            x: {
              ticks: { color: '#94a3b8' },
              grid: { display: false },
            },
            y: {
              ticks: {
                color: '#94a3b8',
                callback: (value) => {
                  const hours = Number(value) / 3600;
                  return `${hours}h`;
                },
              },
              grid: { color: '#334155' },
            },
          },
          plugins: {
            legend: { display: false },
            tooltip: {
              callbacks: {
                label: (context) => {
                  const seconds = context.raw as number;
                  return formatDurationShort(seconds);
                },
              },
            },
          },
        },
      });
    }
  }

  private updateCharts(): void {
    if (!this.report) return;

    // Update task chart
    if (this.taskChart) {
      this.taskChart.data.labels = this.report.task_summaries.map(t => t.task_name);
      this.taskChart.data.datasets[0].data = this.report.task_summaries.map(t => t.total_seconds);
      this.taskChart.data.datasets[0].backgroundColor = this.report.task_summaries.map(t => t.task_color);
      this.taskChart.update();
    }

    // Update daily chart
    if (this.dailyChart) {
      this.dailyChart.data.labels = this.report.daily_summaries.map(d => {
        const date = new Date(d.date);
        return `${date.getMonth() + 1}/${date.getDate()}`;
      });
      this.dailyChart.data.datasets[0].data = this.report.daily_summaries.map(d => d.total_seconds);
      this.dailyChart.update();
    }

    // Update task detail list
    this.updateTaskList();
  }

  private updateSummary(): void {
    if (!this.report) return;

    const totalTime = this.element.querySelector('#total-time');
    const workingDays = this.element.querySelector('#working-days');
    const avgTime = this.element.querySelector('#avg-time');
    const entryCount = this.element.querySelector('#entry-count');

    if (totalTime) totalTime.textContent = formatDurationShort(this.report.total_seconds);
    if (workingDays) workingDays.textContent = `${this.report.working_days}日`;
    if (avgTime) avgTime.textContent = formatDurationShort(this.report.average_seconds_per_day);
    if (entryCount) entryCount.textContent = `${this.report.total_entries}件`;
  }

  private updateTaskList(): void {
    if (!this.report) return;

    const listContainer = this.element.querySelector('.task-detail-list');
    if (!listContainer) return;

    if (this.report.task_summaries.length === 0) {
      listContainer.innerHTML = '<div class="empty-hint">データがありません</div>';
      return;
    }

    listContainer.innerHTML = this.report.task_summaries
      .map(task => {
        const percentage = this.report!.total_seconds > 0
          ? Math.round((task.total_seconds / this.report!.total_seconds) * 100)
          : 0;
        return `
          <div class="task-detail-item">
            <div class="task-detail-color" style="background-color: ${task.task_color}"></div>
            <div class="task-detail-info">
              <div class="task-detail-name">${task.task_name}</div>
              <div class="task-detail-stats">
                ${formatDurationShort(task.total_seconds)} (${percentage}%) / ${task.entry_count}件
              </div>
            </div>
            <div class="task-detail-bar">
              <div class="task-detail-bar-fill" style="width: ${percentage}%; background-color: ${task.task_color}"></div>
            </div>
          </div>
        `;
      })
      .join('');
  }

  private async exportToPng(): Promise<void> {
    const content = document.getElementById('report-content');
    if (!content) return;

    try {
      // Use html2canvas dynamically if available, otherwise use native approach
      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');
      if (!ctx) return;

      // Get the charts as images
      const taskCanvas = document.getElementById('task-chart') as HTMLCanvasElement;
      const dailyCanvas = document.getElementById('daily-chart') as HTMLCanvasElement;

      // Create a combined export
      const width = 1200;
      const height = 800;
      canvas.width = width;
      canvas.height = height;

      // Background
      ctx.fillStyle = '#1e293b';
      ctx.fillRect(0, 0, width, height);

      // Title
      ctx.fillStyle = '#f8fafc';
      ctx.font = 'bold 24px sans-serif';
      ctx.fillText(`${this.selectedYear}年${this.selectedMonth}月 月次レポート`, 40, 50);

      // Summary
      if (this.report) {
        ctx.font = '16px sans-serif';
        ctx.fillStyle = '#94a3b8';
        ctx.fillText(`合計: ${formatDurationShort(this.report.total_seconds)}`, 40, 90);
        ctx.fillText(`稼働日数: ${this.report.working_days}日`, 200, 90);
        ctx.fillText(`平均: ${formatDurationShort(this.report.average_seconds_per_day)}/日`, 360, 90);
        ctx.fillText(`エントリ数: ${this.report.total_entries}件`, 540, 90);
      }

      // Draw task chart
      if (taskCanvas) {
        ctx.drawImage(taskCanvas, 40, 120, 400, 300);
      }

      // Draw daily chart
      if (dailyCanvas) {
        ctx.drawImage(dailyCanvas, 480, 120, 680, 300);
      }

      // Task list
      if (this.report) {
        let y = 460;
        ctx.font = 'bold 16px sans-serif';
        ctx.fillStyle = '#f8fafc';
        ctx.fillText('タスク別詳細', 40, y);
        y += 30;

        ctx.font = '14px sans-serif';
        this.report.task_summaries.forEach((task) => {
          if (y > height - 40) return;
          const percentage = this.report!.total_seconds > 0
            ? Math.round((task.total_seconds / this.report!.total_seconds) * 100)
            : 0;

          // Color dot
          ctx.fillStyle = task.task_color;
          ctx.beginPath();
          ctx.arc(50, y - 4, 6, 0, Math.PI * 2);
          ctx.fill();

          // Task name and stats
          ctx.fillStyle = '#f8fafc';
          ctx.fillText(task.task_name, 70, y);
          ctx.fillStyle = '#94a3b8';
          ctx.fillText(`${formatDurationShort(task.total_seconds)} (${percentage}%)`, 300, y);

          // Progress bar
          ctx.fillStyle = '#334155';
          ctx.fillRect(450, y - 10, 200, 12);
          ctx.fillStyle = task.task_color;
          ctx.fillRect(450, y - 10, percentage * 2, 12);

          y += 28;
        });
      }

      // Download
      const link = document.createElement('a');
      link.download = `report-${this.selectedYear}-${String(this.selectedMonth).padStart(2, '0')}.png`;
      link.href = canvas.toDataURL('image/png');
      link.click();
    } catch (err) {
      console.error('Failed to export PNG:', err);
    }
  }
}
