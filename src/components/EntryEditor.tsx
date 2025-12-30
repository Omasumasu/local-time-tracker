import { useEffect, useState } from 'react';
import { Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useStore } from '@/hooks/useStore';
import { toDatetimeLocal, fromDatetimeLocal } from '@/utils/format';
import type { TimeEntryWithRelations } from '@/types';

interface EntryEditorProps {
  entry: TimeEntryWithRelations | null;
  open: boolean;
  onClose: () => void;
}

export function EntryEditor({ entry, open, onClose }: EntryEditorProps) {
  const { tasks, updateEntry, deleteEntry } = useStore();
  const [taskId, setTaskId] = useState<string>('');
  const [startedAt, setStartedAt] = useState('');
  const [endedAt, setEndedAt] = useState('');
  const [memo, setMemo] = useState('');
  const [isDeleting, setIsDeleting] = useState(false);

  const activeTasks = tasks.filter((t) => !t.archived);

  useEffect(() => {
    if (entry) {
      setTaskId(entry.task_id || '');
      setStartedAt(toDatetimeLocal(entry.started_at));
      setEndedAt(entry.ended_at ? toDatetimeLocal(entry.ended_at) : '');
      setMemo(entry.memo || '');
      setIsDeleting(false);
    }
  }, [entry]);

  const handleSave = async () => {
    if (!entry) return;

    try {
      await updateEntry(entry.id, {
        task_id: taskId || null,
        started_at: fromDatetimeLocal(startedAt),
        ended_at: endedAt ? fromDatetimeLocal(endedAt) : null,
        memo: memo || null,
      });
      onClose();
    } catch (err) {
      console.error('Failed to update entry:', err);
    }
  };

  const handleDelete = async () => {
    if (!entry) return;

    if (!isDeleting) {
      setIsDeleting(true);
      return;
    }

    try {
      await deleteEntry(entry.id);
      onClose();
    } catch (err) {
      console.error('Failed to delete entry:', err);
    }
  };

  if (!entry) return null;

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>時間記録を編集</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">タスク</label>
            <Select value={taskId} onValueChange={setTaskId}>
              <SelectTrigger>
                <SelectValue placeholder="タスクを選択" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">未分類</SelectItem>
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
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">開始時刻</label>
            <Input
              type="datetime-local"
              value={startedAt}
              onChange={(e) => setStartedAt(e.target.value)}
            />
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">終了時刻</label>
            <Input
              type="datetime-local"
              value={endedAt}
              onChange={(e) => setEndedAt(e.target.value)}
              disabled={!entry.ended_at}
            />
            {!entry.ended_at && (
              <p className="text-xs text-muted-foreground">計測中は編集できません</p>
            )}
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">メモ</label>
            <Input
              value={memo}
              onChange={(e) => setMemo(e.target.value)}
              placeholder="メモ（オプション）"
            />
          </div>
        </div>

        <DialogFooter className="flex justify-between sm:justify-between">
          <Button
            variant={isDeleting ? 'destructive' : 'outline'}
            onClick={handleDelete}
          >
            <Trash2 className="w-4 h-4 mr-2" />
            {isDeleting ? '本当に削除' : '削除'}
          </Button>
          <div className="flex gap-2">
            <Button variant="outline" onClick={onClose}>
              キャンセル
            </Button>
            <Button onClick={handleSave}>保存</Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
