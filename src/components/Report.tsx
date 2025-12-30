import { useEffect, useRef, useState } from 'react';
import { Chart, registerables } from 'chart.js';
import { Download } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useStore } from '@/hooks/useStore';
import { formatDurationShort } from '@/utils/format';
import type { MonthlyReport } from '@/types';

Chart.register(...registerables);

export function Report() {
  const { getMonthlyReport, getAvailableMonths } = useStore();
  const [report, setReport] = useState<MonthlyReport | null>(null);
  const [availableMonths, setAvailableMonths] = useState<[number, number][]>([]);
  const [selectedMonth, setSelectedMonth] = useState<string>('');

  const taskChartRef = useRef<HTMLCanvasElement>(null);
  const dailyChartRef = useRef<HTMLCanvasElement>(null);
  const taskChartInstance = useRef<Chart | null>(null);
  const dailyChartInstance = useRef<Chart | null>(null);

  // Load available months
  useEffect(() => {
    const load = async () => {
      try {
        const months = await getAvailableMonths();
        setAvailableMonths(months);
        if (months.length > 0) {
          setSelectedMonth(`${months[0][0]}-${months[0][1]}`);
        }
      } catch (err) {
        console.error('Failed to load available months:', err);
      }
    };
    load();
  }, [getAvailableMonths]);

  // Load report when month changes
  useEffect(() => {
    if (!selectedMonth) return;

    const [year, month] = selectedMonth.split('-').map(Number);
    const load = async () => {
      try {
        const data = await getMonthlyReport(year, month);
        setReport(data);
      } catch (err) {
        console.error('Failed to load report:', err);
      }
    };
    load();
  }, [selectedMonth, getMonthlyReport]);

  // Update charts
  useEffect(() => {
    if (!report) return;

    // Task chart
    if (taskChartRef.current) {
      if (taskChartInstance.current) {
        taskChartInstance.current.destroy();
      }

      taskChartInstance.current = new Chart(taskChartRef.current, {
        type: 'doughnut',
        data: {
          labels: report.task_summaries.map((t) => t.task_name),
          datasets: [
            {
              data: report.task_summaries.map((t) => t.total_seconds),
              backgroundColor: report.task_summaries.map((t) => t.task_color),
              borderWidth: 0,
            },
          ],
        },
        options: {
          responsive: true,
          maintainAspectRatio: true,
          plugins: {
            legend: {
              position: 'right',
              labels: {
                color: 'hsl(var(--foreground))',
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

    // Daily chart
    if (dailyChartRef.current) {
      if (dailyChartInstance.current) {
        dailyChartInstance.current.destroy();
      }

      const maxSeconds = Math.max(...report.daily_summaries.map((d) => d.total_seconds), 0);
      const maxHours = Math.ceil(maxSeconds / 3600);

      dailyChartInstance.current = new Chart(dailyChartRef.current, {
        type: 'bar',
        data: {
          labels: report.daily_summaries.map((d) => {
            const date = new Date(d.date);
            return `${date.getMonth() + 1}/${date.getDate()}`;
          }),
          datasets: [
            {
              label: '稼働時間',
              data: report.daily_summaries.map((d) => d.total_seconds),
              backgroundColor: 'hsl(var(--primary))',
              borderRadius: 4,
            },
          ],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          scales: {
            x: {
              ticks: { color: 'hsl(var(--muted-foreground))' },
              grid: { display: false },
            },
            y: {
              min: 0,
              max: maxHours * 3600,
              ticks: {
                color: 'hsl(var(--muted-foreground))',
                stepSize: 3600,
                callback: (value) => {
                  const hours = Math.floor(Number(value) / 3600);
                  return `${hours}h`;
                },
              },
              grid: { color: 'hsl(var(--border))' },
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

    return () => {
      if (taskChartInstance.current) {
        taskChartInstance.current.destroy();
      }
      if (dailyChartInstance.current) {
        dailyChartInstance.current.destroy();
      }
    };
  }, [report]);

  const handleExportPng = () => {
    if (!report) return;

    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

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
    ctx.fillText(`${report.year}年${report.month}月 月次レポート`, 40, 50);

    // Summary
    ctx.font = '16px sans-serif';
    ctx.fillStyle = '#94a3b8';
    ctx.fillText(`合計: ${formatDurationShort(report.total_seconds)}`, 40, 90);
    ctx.fillText(`稼働日数: ${report.working_days}日`, 200, 90);
    ctx.fillText(`平均: ${formatDurationShort(report.average_seconds_per_day)}/日`, 360, 90);
    ctx.fillText(`エントリ数: ${report.total_entries}件`, 540, 90);

    // Draw charts
    if (taskChartRef.current) {
      ctx.drawImage(taskChartRef.current, 40, 120, 400, 300);
    }
    if (dailyChartRef.current) {
      ctx.drawImage(dailyChartRef.current, 480, 120, 680, 300);
    }

    // Task list
    let y = 460;
    ctx.font = 'bold 16px sans-serif';
    ctx.fillStyle = '#f8fafc';
    ctx.fillText('タスク別詳細', 40, y);
    y += 30;

    ctx.font = '14px sans-serif';
    report.task_summaries.forEach((task) => {
      if (y > height - 40) return;
      const percentage =
        report.total_seconds > 0
          ? Math.round((task.total_seconds / report.total_seconds) * 100)
          : 0;

      ctx.fillStyle = task.task_color;
      ctx.beginPath();
      ctx.arc(50, y - 4, 6, 0, Math.PI * 2);
      ctx.fill();

      ctx.fillStyle = '#f8fafc';
      ctx.fillText(task.task_name, 70, y);
      ctx.fillStyle = '#94a3b8';
      ctx.fillText(`${formatDurationShort(task.total_seconds)} (${percentage}%)`, 300, y);

      ctx.fillStyle = '#334155';
      ctx.fillRect(450, y - 10, 200, 12);
      ctx.fillStyle = task.task_color;
      ctx.fillRect(450, y - 10, percentage * 2, 12);

      y += 28;
    });

    // Download
    const link = document.createElement('a');
    link.download = `report-${report.year}-${String(report.month).padStart(2, '0')}.png`;
    link.href = canvas.toDataURL('image/png');
    link.click();
  };

  if (availableMonths.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <p className="text-muted-foreground">データがありません</p>
          <p className="text-sm text-muted-foreground mt-1">
            時間を記録するとレポートが表示されます
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-semibold">月次レポート</h2>
        <div className="flex items-center gap-3">
          <Select value={selectedMonth} onValueChange={setSelectedMonth}>
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {availableMonths.map(([year, month]) => (
                <SelectItem key={`${year}-${month}`} value={`${year}-${month}`}>
                  {year}年{month}月
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button variant="outline" onClick={handleExportPng}>
            <Download className="w-4 h-4 mr-2" />
            PNG出力
          </Button>
        </div>
      </div>

      {report && (
        <>
          {/* Summary Cards */}
          <div className="grid grid-cols-4 gap-4">
            <Card>
              <CardContent className="pt-6 text-center">
                <div className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  合計時間
                </div>
                <div className="text-2xl font-semibold">
                  {formatDurationShort(report.total_seconds)}
                </div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="pt-6 text-center">
                <div className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  稼働日数
                </div>
                <div className="text-2xl font-semibold">{report.working_days}日</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="pt-6 text-center">
                <div className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  平均時間/日
                </div>
                <div className="text-2xl font-semibold">
                  {formatDurationShort(report.average_seconds_per_day)}
                </div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="pt-6 text-center">
                <div className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  エントリ数
                </div>
                <div className="text-2xl font-semibold">{report.total_entries}件</div>
              </CardContent>
            </Card>
          </div>

          {/* Charts */}
          <div className="grid grid-cols-3 gap-6">
            <Card>
              <CardHeader>
                <CardTitle className="text-sm">タスク別内訳</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64 flex items-center justify-center">
                  <canvas ref={taskChartRef} />
                </div>
              </CardContent>
            </Card>
            <Card className="col-span-2">
              <CardHeader>
                <CardTitle className="text-sm">日別推移</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64">
                  <canvas ref={dailyChartRef} />
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Task Details */}
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">タスク別詳細</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              {report.task_summaries.map((task) => {
                const percentage =
                  report.total_seconds > 0
                    ? Math.round((task.total_seconds / report.total_seconds) * 100)
                    : 0;
                return (
                  <div key={task.task_id || 'uncategorized'} className="flex items-center gap-3">
                    <div
                      className="w-3 h-3 rounded-full shrink-0"
                      style={{ backgroundColor: task.task_color }}
                    />
                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-medium">{task.task_name}</div>
                      <div className="text-xs text-muted-foreground">
                        {formatDurationShort(task.total_seconds)} ({percentage}%) /{' '}
                        {task.entry_count}件
                      </div>
                    </div>
                    <div className="w-40 h-2 bg-muted rounded-full overflow-hidden">
                      <div
                        className="h-full rounded-full transition-all"
                        style={{
                          width: `${percentage}%`,
                          backgroundColor: task.task_color,
                        }}
                      />
                    </div>
                  </div>
                );
              })}
            </CardContent>
          </Card>
        </>
      )}
    </div>
  );
}
