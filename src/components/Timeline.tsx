import { useMemo, useState, useEffect } from 'react';
import { Pencil } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { useStore } from '@/hooks/useStore';
import { formatDuration, formatTime, formatDate, calculateDuration } from '@/utils/format';
import type { TimeEntryWithRelations } from '@/types';

interface TimelineProps {
  onEditEntry: (entry: TimeEntryWithRelations) => void;
}

export function Timeline({ onEditEntry }: TimelineProps) {
  const { entries } = useStore();

  const groupedEntries = useMemo(() => {
    const groups = new Map<string, TimeEntryWithRelations[]>();

    for (const entry of entries) {
      const dateKey = entry.started_at.split('T')[0];
      const existing = groups.get(dateKey) || [];
      groups.set(dateKey, [...existing, entry]);
    }

    return Array.from(groups.entries()).sort((a, b) => b[0].localeCompare(a[0]));
  }, [entries]);

  if (entries.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <p className="text-muted-foreground">まだ記録がありません</p>
          <p className="text-sm text-muted-foreground mt-1">
            タイマーを開始して時間を記録しましょう
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      {groupedEntries.map(([date, dayEntries]) => {
        const totalSeconds = dayEntries.reduce((sum, entry) => {
          return sum + calculateDuration(entry.started_at, entry.ended_at);
        }, 0);

        return (
          <Card key={date}>
            <CardHeader className="py-3 bg-muted/50">
              <div className="flex items-center justify-between">
                <CardTitle className="text-sm font-medium">
                  {formatDate(date + 'T00:00:00')}
                </CardTitle>
                <span className="text-sm text-muted-foreground">
                  {formatDuration(totalSeconds)}
                </span>
              </div>
            </CardHeader>
            <CardContent className="p-0 divide-y">
              {dayEntries.map((entry) => (
                <TimelineEntry
                  key={entry.id}
                  entry={entry}
                  onEdit={() => onEditEntry(entry)}
                />
              ))}
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}

interface TimelineEntryProps {
  entry: TimeEntryWithRelations;
  onEdit: () => void;
}

function TimelineEntry({ entry, onEdit }: TimelineEntryProps) {
  const isRunning = !entry.ended_at;
  const [, setTick] = useState(0);

  useEffect(() => {
    if (!isRunning) return;

    const interval = setInterval(() => {
      setTick((t) => t + 1);
    }, 1000);

    return () => clearInterval(interval);
  }, [isRunning]);

  const duration = calculateDuration(entry.started_at, entry.ended_at);

  return (
    <div
      className={`flex items-start gap-4 p-4 ${
        isRunning ? 'bg-green-500/10' : ''
      }`}
    >
      <div className="min-w-[120px]">
        <div className="text-sm text-muted-foreground">
          {formatTime(entry.started_at)}
          {' - '}
          {entry.ended_at ? formatTime(entry.ended_at) : '...'}
        </div>
        <div className={`font-semibold tabular-nums ${isRunning ? 'text-green-500' : ''}`}>
          {formatDuration(duration)}
        </div>
        {isRunning && (
          <span className="text-xs text-green-500">計測中</span>
        )}
      </div>

      <div className="flex-1 min-w-0 space-y-2">
        {entry.task && (
          <Badge
            variant="secondary"
            className="font-medium"
            style={{
              borderLeftWidth: 3,
              borderLeftColor: entry.task.color,
            }}
          >
            {entry.task.name}
          </Badge>
        )}
        {entry.memo && (
          <p className="text-sm text-muted-foreground">{entry.memo}</p>
        )}
        {entry.artifacts.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {entry.artifacts.map((artifact) => (
              <Badge key={artifact.id} variant="outline" className="text-xs">
                {artifact.name}
              </Badge>
            ))}
          </div>
        )}
      </div>

      <Button
        variant="ghost"
        size="icon"
        className="opacity-50 hover:opacity-100"
        onClick={onEdit}
      >
        <Pencil className="w-4 h-4" />
      </Button>
    </div>
  );
}
