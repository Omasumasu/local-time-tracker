import { useState } from 'react';
import { Plus, Pencil, Archive, ArchiveRestore } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { Separator } from '@/components/ui/separator';
import { useStore } from '@/hooks/useStore';
import type { Task } from '@/types';

export function TaskManager() {
  const { tasks, createTask, updateTask, archiveTask, loadTasks } = useStore();
  const [showArchived, setShowArchived] = useState(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [taskName, setTaskName] = useState('');
  const [taskDescription, setTaskDescription] = useState('');
  const [taskColor, setTaskColor] = useState('#3b82f6');

  const displayedTasks = showArchived
    ? tasks
    : tasks.filter((t) => !t.archived);

  const handleCreate = async () => {
    if (!taskName.trim()) return;

    try {
      await createTask({
        name: taskName.trim(),
        description: taskDescription.trim() || undefined,
        color: taskColor,
      });
      setTaskName('');
      setTaskDescription('');
      setTaskColor('#3b82f6');
      setIsCreating(false);
    } catch (err) {
      console.error('Failed to create task:', err);
    }
  };

  const handleUpdate = async () => {
    if (!editingTask || !taskName.trim()) return;

    try {
      await updateTask(editingTask.id, {
        name: taskName.trim(),
        description: taskDescription.trim() || undefined,
        color: taskColor,
      });
      setEditingTask(null);
    } catch (err) {
      console.error('Failed to update task:', err);
    }
  };

  const handleArchive = async (task: Task) => {
    try {
      await archiveTask(task.id, !task.archived);
      if (showArchived) {
        await loadTasks(true);
      }
    } catch (err) {
      console.error('Failed to archive task:', err);
    }
  };

  const openEditDialog = (task: Task) => {
    setTaskName(task.name);
    setTaskDescription(task.description || '');
    setTaskColor(task.color);
    setEditingTask(task);
  };

  const openCreateDialog = () => {
    setTaskName('');
    setTaskDescription('');
    setTaskColor('#3b82f6');
    setIsCreating(true);
  };

  return (
    <>
      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              タスク
            </CardTitle>
            <Button variant="ghost" size="icon" onClick={openCreateDialog}>
              <Plus className="w-4 h-4" />
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-2">
          {displayedTasks.length === 0 ? (
            <p className="text-sm text-muted-foreground text-center py-4">
              タスクがありません
            </p>
          ) : (
            displayedTasks.map((task) => (
              <div
                key={task.id}
                className={`flex items-center gap-3 p-2 rounded-lg bg-muted/50 ${
                  task.archived ? 'opacity-50' : ''
                }`}
              >
                <div
                  className="w-3 h-3 rounded-full shrink-0"
                  style={{ backgroundColor: task.color }}
                />
                <div className="flex-1 min-w-0">
                  <div className="text-sm font-medium truncate">{task.name}</div>
                  {task.description && (
                    <div className="text-xs text-muted-foreground truncate">
                      {task.description}
                    </div>
                  )}
                </div>
                <div className="flex gap-1">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="w-7 h-7 opacity-60 hover:opacity-100"
                    onClick={() => openEditDialog(task)}
                  >
                    <Pencil className="w-3 h-3" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="w-7 h-7 opacity-60 hover:opacity-100"
                    onClick={() => handleArchive(task)}
                  >
                    {task.archived ? (
                      <ArchiveRestore className="w-3 h-3" />
                    ) : (
                      <Archive className="w-3 h-3" />
                    )}
                  </Button>
                </div>
              </div>
            ))
          )}

          <Separator className="my-3" />

          <label className="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer">
            <input
              type="checkbox"
              checked={showArchived}
              onChange={(e) => {
                setShowArchived(e.target.checked);
                loadTasks(e.target.checked);
              }}
              className="accent-primary"
            />
            アーカイブ済みを表示
          </label>
        </CardContent>
      </Card>

      {/* Create/Edit Dialog */}
      <Dialog
        open={isCreating || !!editingTask}
        onOpenChange={(open) => {
          if (!open) {
            setIsCreating(false);
            setEditingTask(null);
          }
        }}
      >
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>
              {editingTask ? 'タスクを編集' : '新しいタスク'}
            </DialogTitle>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">タスク名</label>
              <Input
                value={taskName}
                onChange={(e) => setTaskName(e.target.value)}
                placeholder="タスク名を入力"
              />
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">説明</label>
              <Input
                value={taskDescription}
                onChange={(e) => setTaskDescription(e.target.value)}
                placeholder="説明（オプション）"
              />
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">カラー</label>
              <Input
                type="color"
                value={taskColor}
                onChange={(e) => setTaskColor(e.target.value)}
                className="h-10 cursor-pointer"
              />
            </div>
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setIsCreating(false);
                setEditingTask(null);
              }}
            >
              キャンセル
            </Button>
            <Button onClick={editingTask ? handleUpdate : handleCreate}>
              {editingTask ? '更新' : '作成'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
