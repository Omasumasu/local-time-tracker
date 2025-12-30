import { useState, useEffect } from 'react';
import { Play, Square } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
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
    <motion.div
      initial={{ opacity: 0, y: -20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
    >
      <Card className="mb-6 overflow-hidden">
        <CardContent className="pt-6">
          <motion.div
            className={`text-5xl font-bold text-center mb-6 font-mono tabular-nums ${
              runningEntry ? 'text-green-500' : 'text-foreground'
            }`}
            animate={{
              scale: runningEntry ? [1, 1.02, 1] : 1,
            }}
            transition={{
              duration: 1,
              repeat: runningEntry ? Infinity : 0,
              ease: "easeInOut",
            }}
          >
            {formatDuration(elapsedSeconds)}
          </motion.div>

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

            <AnimatePresence mode="wait">
              {runningEntry ? (
                <motion.div
                  key="stop"
                  initial={{ scale: 0.8, opacity: 0 }}
                  animate={{ scale: 1, opacity: 1 }}
                  exit={{ scale: 0.8, opacity: 0 }}
                  transition={{ type: "spring", stiffness: 500, damping: 30 }}
                >
                  <Button onClick={handleStop} variant="destructive" size="lg">
                    <Square className="w-4 h-4 mr-2" />
                    停止
                  </Button>
                </motion.div>
              ) : (
                <motion.div
                  key="start"
                  initial={{ scale: 0.8, opacity: 0 }}
                  animate={{ scale: 1, opacity: 1 }}
                  exit={{ scale: 0.8, opacity: 0 }}
                  transition={{ type: "spring", stiffness: 500, damping: 30 }}
                >
                  <Button onClick={handleStart} size="lg">
                    <Play className="w-4 h-4 mr-2" />
                    開始
                  </Button>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        </CardContent>
      </Card>
    </motion.div>
  );
}
