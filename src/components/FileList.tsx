import { useState, useMemo } from "react";
import {
  CheckCircle2,
  Upload,
  Download,
  AlertTriangle,
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
import { ComparisonResultDto, SyncState } from "@/types";

interface FileListProps {
  files: ComparisonResultDto[];
}

type SortField = "path" | "size" | "modified" | "state";
type SortOrder = "asc" | "desc";

const getSyncStateIcon = (state: SyncState) => {
  switch (state) {
    case SyncState.InSync:
      return <CheckCircle2 className="h-4 w-4 text-green-500" />;
    case SyncState.NeedsUpload:
      return <Upload className="h-4 w-4 text-blue-500" />;
    case SyncState.NeedsDownload:
      return <Download className="h-4 w-4 text-blue-500" />;
    case SyncState.Conflict:
      return <AlertTriangle className="h-4 w-4 text-red-500" />;
    default:
      return <File className="h-4 w-4 text-gray-400" />;
  }
};

const getSyncStateBadge = (state: SyncState) => {
  switch (state) {
    case SyncState.InSync:
      return <Badge variant="default">In Sync</Badge>;
    case SyncState.NeedsUpload:
      return <Badge variant="default">Needs Upload</Badge>;
    case SyncState.NeedsDownload:
      return <Badge variant="default">Needs Download</Badge>;
    case SyncState.Conflict:
      return <Badge variant="destructive">Conflict</Badge>;
    default:
      return <Badge variant="outline">Unknown</Badge>;
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
          <option value="all">All Files</option>
          <option value={SyncState.InSync}>In Sync</option>
          <option value={SyncState.NeedsUpload}>Needs Upload</option>
          <option value={SyncState.NeedsDownload}>Needs Download</option>
          <option value={SyncState.Conflict}>Conflict</option>
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
              <TableHead>
                <button
                  className="flex items-center gap-1 hover:underline"
                  onClick={() => handleSort("modified")}
                >
                  Last Modified
                  <ArrowUpDown className="h-4 w-4" />
                </button>
              </TableHead>
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
                <TableCell colSpan={5} className="text-center text-muted-foreground">
                  No files found
                </TableCell>
              </TableRow>
            ) : (
              filteredAndSortedFiles.map((file) => (
                <TableRow key={file.path}>
                  <TableCell>{getSyncStateIcon(file.state)}</TableCell>
                  <TableCell className="font-medium">{file.path}</TableCell>
                  <TableCell>
                    {formatFileSize(file.local_size ?? file.remote_size)}
                  </TableCell>
                  <TableCell>
                    {formatDate(file.local_modified ?? file.remote_modified)}
                  </TableCell>
                  <TableCell>{getSyncStateBadge(file.state)}</TableCell>
                </TableRow>
              ))
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
