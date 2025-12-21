import { useState, useMemo } from "react";
import {
  CheckCircle2,
  ArrowUp,
  ArrowDown,
  AlertTriangle,
  XCircle,
  Ban,
  Trash2,
  Loader2,
  File,
  ArrowUpDown,
} from "lucide-react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { ComparisonResultDto, SyncState } from "@/types";

interface FileListProps {
  files: ComparisonResultDto[];
}

type SortField = "path" | "size" | "modified" | "state";
type SortOrder = "asc" | "desc";

const getSyncStateIcon = (state: SyncState) => {
  switch (state) {
    case SyncState.InSync:
      return <CheckCircle2 className="h-4 w-4" style={{ color: "#22c55e" }} />;
    case SyncState.Syncing:
      return <Loader2 className="h-4 w-4 animate-spin" style={{ color: "#3b82f6" }} />;
    case SyncState.NeedsUpload:
      return <ArrowUp className="h-4 w-4" style={{ color: "#f59e0b" }} />;
    case SyncState.NeedsDownload:
      return <ArrowDown className="h-4 w-4" style={{ color: "#f59e0b" }} />;
    case SyncState.Conflict:
      return <AlertTriangle className="h-4 w-4" style={{ color: "#eab308" }} />;
    case SyncState.Error:
      return <XCircle className="h-4 w-4" style={{ color: "#ef4444" }} />;
    case SyncState.Excluded:
      return <Ban className="h-4 w-4" style={{ color: "#9ca3af" }} />;
    case SyncState.PendingDelete:
      return <Trash2 className="h-4 w-4" style={{ color: "#6b7280" }} />;
    default:
      return <File className="h-4 w-4 text-gray-400" />;
  }
};

const getSyncStateBadge = (state: SyncState) => {
  switch (state) {
    case SyncState.InSync:
      return <Badge className="bg-green-500 hover:bg-green-600">同期済み</Badge>;
    case SyncState.Syncing:
      return <Badge className="bg-blue-500 hover:bg-blue-600">同期中</Badge>;
    case SyncState.NeedsUpload:
      return <Badge className="bg-orange-500 hover:bg-orange-600">アップロード待ち</Badge>;
    case SyncState.NeedsDownload:
      return <Badge className="bg-orange-500 hover:bg-orange-600">ダウンロード待ち</Badge>;
    case SyncState.Conflict:
      return <Badge className="bg-yellow-500 hover:bg-yellow-600">競合</Badge>;
    case SyncState.Error:
      return <Badge className="bg-red-500 hover:bg-red-600">エラー</Badge>;
    case SyncState.Excluded:
      return <Badge className="bg-gray-500 hover:bg-gray-600">除外</Badge>;
    case SyncState.PendingDelete:
      return <Badge className="bg-gray-600 hover:bg-gray-700">削除待ち</Badge>;
    default:
      return <Badge variant="outline">不明</Badge>;
  }
};

const formatFileSize = (bytes?: number): string => {
  if (bytes === undefined || bytes === null) return "-";
  if (bytes === 0) return "0 B";

  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
};

const formatDate = (dateString?: string): string => {
  if (!dateString) return "-";
  try {
    const date = new Date(dateString);
    return date.toLocaleString();
  } catch {
    return "-";
  }
};

const getTooltipContent = (file: ComparisonResultDto): string => {
  const localModified = file.local_modified ? formatDate(file.local_modified) : "-";
  const remoteModified = file.remote_modified ? formatDate(file.remote_modified) : "-";

  switch (file.state) {
    case SyncState.InSync:
      return `ローカルとクラウドが一致しています\n最終同期: ${localModified}`;
    case SyncState.Syncing:
      return `ファイルを同期中です...`;
    case SyncState.NeedsUpload:
      return `ローカルが新しいファイルです\nローカル: ${localModified}\nクラウド: ${remoteModified}\n次回同期でアップロードされます`;
    case SyncState.NeedsDownload:
      return `クラウドが新しいファイルです\nローカル: ${localModified}\nクラウド: ${remoteModified}\n次回同期でダウンロードされます`;
    case SyncState.Conflict:
      return `両方で異なる変更が行われています\nローカル: ${localModified}\nクラウド: ${remoteModified}\n同期時に両方保存されます`;
    case SyncState.Error:
      return `同期に失敗しました\n「再試行」ボタンで再度同期を試みる`;
    case SyncState.Excluded:
      return `同期対象外のファイルです`;
    case SyncState.PendingDelete:
      return `ローカルで削除されました\n削除日時: ${localModified}\n「復元」で取り消せます`;
    default:
      return "";
  }
};

export function FileList({ files }: FileListProps) {
  const [sortField, setSortField] = useState<SortField>("path");
  const [sortOrder, setSortOrder] = useState<SortOrder>("asc");
  const [filterText, setFilterText] = useState("");
  const [filterState, setFilterState] = useState<SyncState | "all">("all");

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortOrder(sortOrder === "asc" ? "desc" : "asc");
    } else {
      setSortField(field);
      setSortOrder("asc");
    }
  };

  const filteredAndSortedFiles = useMemo(() => {
    let result = [...files];

    // Apply text filter
    if (filterText) {
      result = result.filter((file) =>
        file.path.toLowerCase().includes(filterText.toLowerCase())
      );
    }

    // Apply state filter
    if (filterState !== "all") {
      result = result.filter((file) => file.state === filterState);
    }

    // Apply sorting
    result.sort((a, b) => {
      let compareValue = 0;

      switch (sortField) {
        case "path":
          compareValue = a.path.localeCompare(b.path);
          break;
        case "size": {
          const sizeA = a.local_size ?? a.remote_size ?? 0;
          const sizeB = b.local_size ?? b.remote_size ?? 0;
          compareValue = sizeA - sizeB;
          break;
        }
        case "modified": {
          const dateA = a.local_modified ?? a.remote_modified ?? "";
          const dateB = b.local_modified ?? b.remote_modified ?? "";
          compareValue = dateA.localeCompare(dateB);
          break;
        }
        case "state":
          compareValue = a.state.localeCompare(b.state);
          break;
      }

      return sortOrder === "asc" ? compareValue : -compareValue;
    });

    return result;
  }, [files, filterText, filterState, sortField, sortOrder]);

  return (
    <div className="space-y-4">
      {/* Filters */}
      <div className="flex gap-4">
        <div className="flex-1">
          <Input
            placeholder="Filter by filename..."
            value={filterText}
            onChange={(e) => setFilterText(e.target.value)}
          />
        </div>
        <select
          className="rounded-md border border-input bg-background px-3 py-2"
          value={filterState}
          onChange={(e) => setFilterState(e.target.value as SyncState | "all")}
        >
          <option value="all">すべて</option>
          <option value={SyncState.InSync}>同期済み</option>
          <option value={SyncState.Syncing}>同期中</option>
          <option value={SyncState.NeedsUpload}>アップロード待ち</option>
          <option value={SyncState.NeedsDownload}>ダウンロード待ち</option>
          <option value={SyncState.Conflict}>競合</option>
          <option value={SyncState.Error}>エラー</option>
          <option value={SyncState.Excluded}>除外</option>
          <option value={SyncState.PendingDelete}>削除待ち</option>
        </select>
      </div>

      {/* File list table */}
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-12">Status</TableHead>
              <TableHead>
                <button
                  className="flex items-center gap-1 hover:underline"
                  onClick={() => handleSort("path")}
                >
                  File Path
                  <ArrowUpDown className="h-4 w-4" />
                </button>
              </TableHead>
              <TableHead>
                <button
                  className="flex items-center gap-1 hover:underline"
                  onClick={() => handleSort("size")}
                >
                  Size
                  <ArrowUpDown className="h-4 w-4" />
                </button>
              </TableHead>
              <TableHead>Local Modified</TableHead>
              <TableHead>Remote Modified</TableHead>
              <TableHead>
                <button
                  className="flex items-center gap-1 hover:underline"
                  onClick={() => handleSort("state")}
                >
                  Sync State
                  <ArrowUpDown className="h-4 w-4" />
                </button>
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {filteredAndSortedFiles.length === 0 ? (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground">
                  No files found
                </TableCell>
              </TableRow>
            ) : (
              filteredAndSortedFiles.map((file) => {
                // 背景色の設定（競合とエラー）
                const bgColor =
                  file.state === SyncState.Conflict ? "#fef9c3" :
                  file.state === SyncState.Error ? "#fee2e2" :
                  undefined;

                // グレーアウト（削除待ちと除外）
                const isGrayedOut =
                  file.state === SyncState.PendingDelete ||
                  file.state === SyncState.Excluded;

                const textColor = isGrayedOut ? "#9ca3af" : undefined;
                const textDecoration = file.state === SyncState.PendingDelete ? "line-through" : undefined;
                const fontSize = file.state === SyncState.Excluded ? "0.9em" : undefined;

                return (
                  <TooltipProvider key={file.path}>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <TableRow style={{ backgroundColor: bgColor }}>
                          <TableCell>{getSyncStateIcon(file.state)}</TableCell>
                          <TableCell
                            className="font-medium"
                            style={{
                              color: textColor,
                              textDecoration,
                              fontSize
                            }}
                          >
                            {file.path}
                          </TableCell>
                          <TableCell style={{ color: textColor }}>
                            {formatFileSize(file.local_size ?? file.remote_size)}
                          </TableCell>
                          <TableCell style={{ color: textColor }}>
                            {formatDate(file.local_modified)}
                          </TableCell>
                          <TableCell style={{ color: textColor }}>
                            {formatDate(file.remote_modified)}
                          </TableCell>
                          <TableCell>{getSyncStateBadge(file.state)}</TableCell>
                        </TableRow>
                      </TooltipTrigger>
                      <TooltipContent className="whitespace-pre-line">
                        {getTooltipContent(file)}
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                );
              })
            )}
          </TableBody>
        </Table>
      </div>

      {/* Summary */}
      <div className="text-sm text-muted-foreground">
        Showing {filteredAndSortedFiles.length} of {files.length} files
      </div>
    </div>
  );
}
