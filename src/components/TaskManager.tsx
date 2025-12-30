import { useState } from 'react';
import {
  Plus, Pencil, Archive, ArchiveRestore, Folder, FolderPlus, ChevronRight, ChevronDown, Trash2,
  Briefcase, Code, FileText, Book, Music, Image, Video, Mail, Calendar, Star, Heart, Settings,
  Home, User, Users, Globe, Zap, Coffee, Gamepad2, Palette, Camera
} from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Separator } from '@/components/ui/separator';
import { useStore } from '@/hooks/useStore';
import type { Task, Folder as FolderType } from '@/types';

// Available icons for folders
const FOLDER_ICONS: Record<string, LucideIcon> = {
  folder: Folder,
  briefcase: Briefcase,
  code: Code,
  fileText: FileText,
  book: Book,
  music: Music,
  image: Image,
  video: Video,
  mail: Mail,
  calendar: Calendar,
  star: Star,
  heart: Heart,
  settings: Settings,
  home: Home,
  user: User,
  users: Users,
  globe: Globe,
  zap: Zap,
  coffee: Coffee,
  gamepad: Gamepad2,
  palette: Palette,
  camera: Camera,
};

export function TaskManager() {
  const {
    tasks,
    folders,
    createTask,
    updateTask,
    archiveTask,
    loadTasks,
    createFolder,
    updateFolder,
    deleteFolder,
  } = useStore();

  const [showArchived, setShowArchived] = useState(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [taskName, setTaskName] = useState('');
  const [taskDescription, setTaskDescription] = useState('');
  const [taskColor, setTaskColor] = useState('#3b82f6');
  const [taskFolderId, setTaskFolderId] = useState<string | null>(null);

  // Folder state
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());
  const [editingFolder, setEditingFolder] = useState<FolderType | null>(null);
  const [isCreatingFolder, setIsCreatingFolder] = useState(false);
  const [folderName, setFolderName] = useState('');
  const [folderColor, setFolderColor] = useState('#6b7280');
  const [deletingFolder, setDeletingFolder] = useState<FolderType | null>(null);
  const [folderIcon, setFolderIcon] = useState<string>('folder');

  const displayedTasks = showArchived
    ? tasks
    : tasks.filter((t) => !t.archived);

  const getTasksByFolderId = (folderId: string | null) => {
    return displayedTasks.filter((t) => t.folder_id === folderId);
  };

  const toggleFolder = (folderId: string) => {
    setExpandedFolders((prev) => {
      const next = new Set(prev);
      if (next.has(folderId)) {
        next.delete(folderId);
      } else {
        next.add(folderId);
      }
      return next;
    });
  };

  const handleCreate = async () => {
    if (!taskName.trim()) return;

    try {
      await createTask({
        name: taskName.trim(),
        description: taskDescription.trim() || undefined,
        color: taskColor,
        folder_id: taskFolderId || undefined,
      });
      setTaskName('');
      setTaskDescription('');
      setTaskColor('#3b82f6');
      setTaskFolderId(null);
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
        folder_id: taskFolderId,
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
    setTaskFolderId(task.folder_id);
    setEditingTask(task);
  };

  const openCreateDialog = (folderId?: string) => {
    setTaskName('');
    setTaskDescription('');
    setTaskColor('#3b82f6');
    setTaskFolderId(folderId || null);
    setIsCreating(true);
  };

  // Folder handlers
  const handleCreateFolder = async () => {
    if (!folderName.trim()) return;

    try {
      const newFolder = await createFolder({
        name: folderName.trim(),
        color: folderColor,
        icon: folderIcon,
      });
      setExpandedFolders((prev) => new Set(prev).add(newFolder.id));
      setFolderName('');
      setFolderColor('#6b7280');
      setFolderIcon('folder');
      setIsCreatingFolder(false);
    } catch (err) {
      console.error('Failed to create folder:', err);
    }
  };

  const handleUpdateFolder = async () => {
    if (!editingFolder || !folderName.trim()) return;

    try {
      await updateFolder(editingFolder.id, {
        name: folderName.trim(),
        color: folderColor,
        icon: folderIcon,
      });
      setEditingFolder(null);
    } catch (err) {
      console.error('Failed to update folder:', err);
    }
  };

  const handleDeleteFolder = async () => {
    if (!deletingFolder) return;

    try {
      await deleteFolder(deletingFolder.id);
      setDeletingFolder(null);
    } catch (err) {
      console.error('Failed to delete folder:', err);
    }
  };

  const openEditFolderDialog = (folder: FolderType, e: React.MouseEvent) => {
    e.stopPropagation();
    setFolderName(folder.name);
    setFolderColor(folder.color);
    setFolderIcon(folder.icon || 'folder');
    setEditingFolder(folder);
  };

  const openCreateFolderDialog = () => {
    setFolderName('');
    setFolderColor('#6b7280');
    setFolderIcon('folder');
    setIsCreatingFolder(true);
  };

  const unassignedTasks = getTasksByFolderId(null);

  return (
    <>
      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              フォルダ &amp; タスク
            </CardTitle>
            <div className="flex gap-1">
              <Button variant="ghost" size="icon" onClick={openCreateFolderDialog} title="フォルダ作成">
                <FolderPlus className="w-4 h-4" />
              </Button>
              <Button variant="ghost" size="icon" onClick={() => openCreateDialog()} title="タスク作成">
                <Plus className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-1">
          {/* Folders */}
          {folders.map((folder) => {
            const folderTasks = getTasksByFolderId(folder.id);
            const isExpanded = expandedFolders.has(folder.id);
            const IconComponent = FOLDER_ICONS[folder.icon || 'folder'] || Folder;

            return (
              <div key={folder.id}>
                <div
                  className="flex items-center gap-2 p-2 rounded-lg hover:bg-muted/50 cursor-pointer group"
                  onClick={() => toggleFolder(folder.id)}
                >
                  {isExpanded ? (
                    <ChevronDown className="w-4 h-4 text-muted-foreground" />
                  ) : (
                    <ChevronRight className="w-4 h-4 text-muted-foreground" />
                  )}
                  <IconComponent
                    className="w-4 h-4"
                    style={{ color: folder.color }}
                  />
                  <span className="flex-1 text-sm font-medium truncate">
                    {folder.name}
                  </span>
                  <span className="text-xs text-muted-foreground">
                    {folderTasks.length}
                  </span>
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="w-6 h-6"
                      onClick={(e) => {
                        e.stopPropagation();
                        openCreateDialog(folder.id);
                      }}
                      title="タスク追加"
                    >
                      <Plus className="w-3 h-3" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="w-6 h-6"
                      onClick={(e) => openEditFolderDialog(folder, e)}
                      title="編集"
                    >
                      <Pencil className="w-3 h-3" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="w-6 h-6 text-destructive hover:text-destructive"
                      onClick={(e) => {
                        e.stopPropagation();
                        setDeletingFolder(folder);
                      }}
                      title="削除"
                    >
                      <Trash2 className="w-3 h-3" />
                    </Button>
                  </div>
                </div>
                {isExpanded && (
                  <div className="ml-6 space-y-1 mt-1">
                    {folderTasks.length === 0 ? (
                      <p className="text-xs text-muted-foreground py-2 pl-2">
                        タスクなし
                      </p>
                    ) : (
                      folderTasks.map((task) => (
                        <TaskItem
                          key={task.id}
                          task={task}
                          onEdit={() => openEditDialog(task)}
                          onArchive={() => handleArchive(task)}
                        />
                      ))
                    )}
                  </div>
                )}
              </div>
            );
          })}

          {/* Unassigned Tasks */}
          {unassignedTasks.length > 0 && (
            <>
              <Separator className="my-2" />
              <div className="text-xs text-muted-foreground uppercase tracking-wider px-2 py-1">
                未分類
              </div>
              <div className="space-y-1">
                {unassignedTasks.map((task) => (
                  <TaskItem
                    key={task.id}
                    task={task}
                    onEdit={() => openEditDialog(task)}
                    onArchive={() => handleArchive(task)}
                  />
                ))}
              </div>
            </>
          )}

          {folders.length === 0 && displayedTasks.length === 0 && (
            <p className="text-sm text-muted-foreground text-center py-4">
              フォルダまたはタスクを作成してください
            </p>
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

      {/* Create/Edit Task Dialog */}
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
              <label className="text-sm font-medium">フォルダ</label>
              <Select
                value={taskFolderId || 'none'}
                onValueChange={(value) => setTaskFolderId(value === 'none' ? null : value)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="フォルダを選択" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">未分類</SelectItem>
                  {folders.map((folder) => (
                    <SelectItem key={folder.id} value={folder.id}>
                      <div className="flex items-center gap-2">
                        <div
                          className="w-3 h-3 rounded-sm"
                          style={{ backgroundColor: folder.color }}
                        />
                        {folder.name}
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
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

      {/* Create/Edit Folder Dialog */}
      <Dialog
        open={isCreatingFolder || !!editingFolder}
        onOpenChange={(open) => {
          if (!open) {
            setIsCreatingFolder(false);
            setEditingFolder(null);
          }
        }}
      >
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>
              {editingFolder ? 'フォルダを編集' : '新しいフォルダ'}
            </DialogTitle>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">フォルダ名</label>
              <Input
                value={folderName}
                onChange={(e) => setFolderName(e.target.value)}
                placeholder="フォルダ名を入力"
              />
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">アイコン</label>
              <div className="grid grid-cols-8 gap-1 p-2 border rounded-lg bg-muted/30">
                {Object.entries(FOLDER_ICONS).map(([key, Icon]) => (
                  <button
                    key={key}
                    type="button"
                    onClick={() => setFolderIcon(key)}
                    className={`p-2 rounded hover:bg-muted transition-colors ${
                      folderIcon === key ? 'bg-primary text-primary-foreground' : ''
                    }`}
                  >
                    <Icon className="w-4 h-4" />
                  </button>
                ))}
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">カラー</label>
              <Input
                type="color"
                value={folderColor}
                onChange={(e) => setFolderColor(e.target.value)}
                className="h-10 cursor-pointer"
              />
            </div>
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setIsCreatingFolder(false);
                setEditingFolder(null);
              }}
            >
              キャンセル
            </Button>
            <Button onClick={editingFolder ? handleUpdateFolder : handleCreateFolder}>
              {editingFolder ? '更新' : '作成'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Folder Confirmation */}
      <AlertDialog open={!!deletingFolder} onOpenChange={(open) => !open && setDeletingFolder(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>フォルダを削除しますか？</AlertDialogTitle>
            <AlertDialogDescription>
              フォルダ「{deletingFolder?.name}」を削除します。
              フォルダ内のタスクは未分類に移動されます。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>キャンセル</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteFolder} className="bg-destructive text-destructive-foreground hover:bg-destructive/90">
              削除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

// Task Item Component
function TaskItem({
  task,
  onEdit,
  onArchive,
}: {
  task: Task;
  onEdit: () => void;
  onArchive: () => void;
}) {
  return (
    <div
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
          onClick={onEdit}
        >
          <Pencil className="w-3 h-3" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="w-7 h-7 opacity-60 hover:opacity-100"
          onClick={onArchive}
        >
          {task.archived ? (
            <ArchiveRestore className="w-3 h-3" />
          ) : (
            <Archive className="w-3 h-3" />
          )}
        </Button>
      </div>
    </div>
  );
}
