import { useEffect, useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Timer } from '@/components/Timer';
import { Timeline } from '@/components/Timeline';
import { TaskManager } from '@/components/TaskManager';
import { Report } from '@/components/Report';
import { Settings } from '@/components/Settings';
import { EntryEditor } from '@/components/EntryEditor';
import { useStore } from '@/hooks/useStore';
import type { TimeEntryWithRelations } from '@/types';

export default function App() {
  const { initialize, isLoading } = useStore();
  const [editingEntry, setEditingEntry] = useState<TimeEntryWithRelations | null>(null);

  useEffect(() => {
    initialize();
  }, [initialize]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-background">
        <div className="text-muted-foreground">読み込み中...</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background text-foreground dark">
      <Tabs defaultValue="tracker" className="h-screen flex flex-col">
        <header className="border-b border-border bg-card px-6 py-3 flex items-center justify-between shrink-0">
          <h1 className="text-lg font-semibold text-foreground">Time Tracker</h1>
          <TabsList className="bg-muted/50 border border-border">
            <TabsTrigger value="tracker" className="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">
              タイムトラッカー
            </TabsTrigger>
            <TabsTrigger value="report" className="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">
              レポート
            </TabsTrigger>
            <TabsTrigger value="settings" className="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">
              設定
            </TabsTrigger>
          </TabsList>
        </header>

        <TabsContent value="tracker" className="flex-1 flex overflow-hidden m-0 data-[state=inactive]:hidden">
          <main className="flex-1 p-6 overflow-y-auto">
            <Timer />
            <Timeline onEditEntry={setEditingEntry} />
          </main>
          <aside className="w-72 border-l bg-card p-4 overflow-y-auto">
            <TaskManager />
          </aside>
        </TabsContent>

        <TabsContent value="report" className="flex-1 overflow-y-auto m-0 p-6 data-[state=inactive]:hidden">
          <Report />
        </TabsContent>

        <TabsContent value="settings" className="flex-1 overflow-y-auto m-0 p-6 data-[state=inactive]:hidden">
          <div className="max-w-xl mx-auto">
            <Settings />
          </div>
        </TabsContent>
      </Tabs>

      <EntryEditor
        entry={editingEntry}
        open={!!editingEntry}
        onClose={() => setEditingEntry(null)}
      />
    </div>
  );
}
