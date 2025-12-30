import { useState } from 'react';
import { Download, Upload, FileSpreadsheet } from 'lucide-react';
import { save, open } from '@tauri-apps/plugin-dialog';
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { useStore } from '@/hooks/useStore';
import type { ExportData } from '@/types';

export function Settings() {
  const { exportData, importData, exportParquet } = useStore();
  const [isExporting, setIsExporting] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const showMessage = (type: 'success' | 'error', text: string) => {
    setMessage({ type, text });
    setTimeout(() => setMessage(null), 3000);
  };

  const handleExportJson = async () => {
    setIsExporting(true);
    try {
      const data = await exportData();
      const json = JSON.stringify(data, null, 2);

      const path = await save({
        defaultPath: `time-tracker-export-${new Date().toISOString().split('T')[0]}.json`,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      });

      if (path) {
        await writeTextFile(path, json);
        showMessage('success', 'エクスポートが完了しました');
      }
    } catch (err) {
      console.error('Export failed:', err);
      showMessage('error', 'エクスポートに失敗しました');
    } finally {
      setIsExporting(false);
    }
  };

  const handleImportJson = async (merge: boolean) => {
    setIsImporting(true);
    try {
      const path = await open({
        filters: [{ name: 'JSON', extensions: ['json'] }],
        multiple: false,
      });

      if (path) {
        const content = await readTextFile(path as string);
        const data: ExportData = JSON.parse(content);
        const result = await importData(data, merge);
        showMessage(
          'success',
          `インポート完了: タスク${result.tasks_imported}件, エントリ${result.entries_imported}件, 成果物${result.artifacts_imported}件`
        );
      }
    } catch (err) {
      console.error('Import failed:', err);
      showMessage('error', 'インポートに失敗しました');
    } finally {
      setIsImporting(false);
    }
  };

  const handleExportParquet = async () => {
    setIsExporting(true);
    try {
      const path = await open({
        directory: true,
        multiple: false,
      });

      if (path) {
        const files = await exportParquet(path as string);
        showMessage('success', `Parquetファイルを出力しました: ${files.length}件`);
      }
    } catch (err) {
      console.error('Parquet export failed:', err);
      showMessage('error', 'Parquetエクスポートに失敗しました');
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <div className="space-y-6">
      {message && (
        <div
          className={`p-4 rounded-lg ${
            message.type === 'success'
              ? 'bg-green-500/10 text-green-500 border border-green-500/20'
              : 'bg-destructive/10 text-destructive border border-destructive/20'
          }`}
        >
          {message.text}
        </div>
      )}

      <Card>
        <CardHeader>
          <CardTitle>データのエクスポート</CardTitle>
          <CardDescription>
            すべてのデータをJSON形式でエクスポートします
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button onClick={handleExportJson} disabled={isExporting}>
            <Download className="w-4 h-4 mr-2" />
            JSONエクスポート
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>データのインポート</CardTitle>
          <CardDescription>
            JSONファイルからデータをインポートします
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <Button
            onClick={() => handleImportJson(true)}
            disabled={isImporting}
            variant="outline"
            className="w-full justify-start"
          >
            <Upload className="w-4 h-4 mr-2" />
            マージインポート（既存データを保持）
          </Button>
          <Button
            onClick={() => handleImportJson(false)}
            disabled={isImporting}
            variant="outline"
            className="w-full justify-start text-destructive hover:text-destructive"
          >
            <Upload className="w-4 h-4 mr-2" />
            置換インポート（既存データを削除）
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Parquetエクスポート</CardTitle>
          <CardDescription>
            データ分析用にParquet形式でエクスポートします
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button onClick={handleExportParquet} disabled={isExporting} variant="outline">
            <FileSpreadsheet className="w-4 h-4 mr-2" />
            Parquetエクスポート
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
