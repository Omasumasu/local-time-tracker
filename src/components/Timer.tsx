import { useState, useEffect } from 'react';
import { Play, Square } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Card, CardContent } from '@/components/ui/card';
import { useStore } from '@/hooks/useStore';
import { formatDuration, calculateDuration } from '@/utils/format';

export function Timer() {
  const { runningEntry, tasks, startEntry, stopEntry } = useStore();
  const [selectedTaskId, setSelectedTaskId] = useState<string>('');
  const [memo, setMemo] = useState('');
  const [, setTick] = useState(0);

  // Timer tick for updating display
  useEffect(() => {
    if (!runningEntry) return;

    const interval = setInterval(() => {
      setTick((t) => t + 1);
    }, 1000);

    return () => clearInterval(interval);
  }, [runningEntry]);

  const activeTasks = tasks.filter((t) => !t.archived);

  const handleStart = async () => {
    try {
      await startEntry(selectedTaskId || undefined, memo || undefined);
      setMemo('');
    } catch (err) {
      console.error('Failed to start entry:', err);
    }
  };

  const handleStop = async () => {
    try {
      await stopEntry(memo || undefined);
      setMemo('');
    } catch (err) {
      console.error('Failed to stop entry:', err);
    }
  };

  const elapsedSeconds = runningEntry
    ? calculateDuration(runningEntry.started_at, null)
    : 0;

  return (
    <Card className="mb-6">
      <CardContent className="pt-6">
        <div
          className={`text-5xl font-bold text-center mb-6 font-mono tabular-nums ${
            runningEntry ? 'text-green-500' : 'text-foreground'
          }`}
        >
          {formatDuration(elapsedSeconds)}
        </div>

        <div className="flex gap-3 items-center">
          <Select
            value={runningEntry?.task_id || selectedTaskId}
            onValueChange={setSelectedTaskId}
            disabled={!!runningEntry}
          >
            <SelectTrigger className="flex-1">
              <SelectValue placeholder="タスクを選択" />
            </SelectTrigger>
            <SelectContent>
              {activeTasks.map((task) => (
                <SelectItem key={task.id} value={task.id}>
                  <div className="flex items-center gap-2">
                    <div
                      className="w-3 h-3 rounded-full"
                      style={{ backgroundColor: task.color }}
                    />
                    {task.name}
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Input
            placeholder="メモ..."
            value={memo}
            onChange={(e) => setMemo(e.target.value)}
            className="flex-1"
          />

          {runningEntry ? (
            <Button onClick={handleStop} variant="destructive" size="lg">
              <Square className="w-4 h-4 mr-2" />
              停止
            </Button>
          ) : (
            <Button onClick={handleStart} size="lg">
              <Play className="w-4 h-4 mr-2" />
              開始
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
