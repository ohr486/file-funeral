import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Folder, RefreshCw, Settings } from "lucide-react";
import { FileList } from "@/components/FileList";
import { SyncButton } from "@/components/SyncButton";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import { SyncStatusResponse } from "@/types";

interface MainPageProps {
  onSettingsClick?: () => void;
}

export function MainPage({ onSettingsClick }: MainPageProps) {
  const [localPath, setLocalPath] = useState("");
  const [remotePrefix, setRemotePrefix] = useState("");
  const [syncStatus, setSyncStatus] = useState<SyncStatusResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // Load saved folder path on component mount
  useEffect(() => {
    const savedSyncFolder = localStorage.getItem("syncFolder");
    if (savedSyncFolder) {
      setLocalPath(savedSyncFolder);
      // Use the last folder name as the remote prefix
      // Remove trailing slashes and filter out empty parts
      const pathParts = savedSyncFolder
        .replace(/[\\/]+$/, "") // Remove trailing slashes
        .split(/[\\/]/)
        .filter(part => part.length > 0);
      const folderName = pathParts.length > 0 ? pathParts[pathParts.length - 1] : "files";
      setRemotePrefix(folderName + "/");
    }
  }, []);

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select folder to sync",
      });

      if (selected && typeof selected === "string") {
        setLocalPath(selected);
        // Use the last folder name as the remote prefix
        // Remove trailing slashes and filter out empty parts
        const pathParts = selected
          .replace(/[\\/]+$/, "") // Remove trailing slashes
          .split(/[\\/]/)
          .filter(part => part.length > 0);
        const folderName = pathParts.length > 0 ? pathParts[pathParts.length - 1] : "files";
        setRemotePrefix(folderName + "/");
        // Save to localStorage
        localStorage.setItem("syncFolder", selected);
        toast.success("Folder selected", {
          description: selected,
        });
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      toast.error("Failed to select folder", {
        description: errorMessage,
      });
    }
  };

  const loadSyncStatus = useCallback(async () => {
    if (!localPath) {
      return;
    }

    setIsLoading(true);
    try {
      const status = await invoke<SyncStatusResponse>("get_sync_status", {
        localPath,
        remotePrefix,
      });

      setSyncStatus(status);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      toast.error("Failed to load sync status", {
        description: errorMessage,
      });
      // Set empty status on error
      setSyncStatus({
        comparisons: [],
        in_sync_count: 0,
        needs_upload_count: 0,
        needs_download_count: 0,
        conflict_count: 0,
        pending_local_deletion_count: 0,
        pending_remote_deletion_count: 0,
      });
    } finally {
      setIsLoading(false);
    }
  }, [localPath, remotePrefix]);

  const handleRefresh = () => {
    loadSyncStatus();
  };

  const handleSyncComplete = () => {
    loadSyncStatus();
  };

  useEffect(() => {
    if (localPath) {
      loadSyncStatus();
    }
  }, [localPath, loadSyncStatus]);

  return (
    <div className="container mx-auto p-6 max-w-7xl">
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">file-funeral</h1>
            <p className="text-muted-foreground">
              Cloud file synchronization application
            </p>
          </div>
          {onSettingsClick && (
            <Button onClick={onSettingsClick} variant="outline">
              <Settings className="mr-2 h-4 w-4" />
              Settings
            </Button>
          )}
        </div>

        {/* Folder selection and sync controls */}
        <div className="flex gap-4 p-4 border rounded-lg bg-card">
          <div className="flex-1 space-y-2">
            <label className="text-sm font-medium">Local Folder</label>
            <div className="flex gap-2">
              <Button
                onClick={handleSelectFolder}
                variant="outline"
                className="w-full justify-start"
              >
                <Folder className="mr-2 h-4 w-4" />
                {localPath || "Select folder to sync"}
              </Button>
              {localPath && (
                <Button
                  onClick={handleRefresh}
                  variant="outline"
                  size="icon"
                  disabled={isLoading}
                >
                  <RefreshCw
                    className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`}
                  />
                </Button>
              )}
            </div>
            {localPath && (
              <p className="text-xs text-muted-foreground">
                Remote prefix: {remotePrefix}
              </p>
            )}
          </div>
          <div className="w-48 space-y-2">
            <label className="text-sm font-medium">Actions</label>
            <SyncButton
              localPath={localPath}
              remotePrefix={remotePrefix}
              onSyncComplete={handleSyncComplete}
              disabled={!localPath}
            />
          </div>
        </div>

        {/* Sync status summary */}
        {syncStatus && (
          <div className="grid gap-4 grid-cols-2 md:grid-cols-4">
            <div className="p-4 border rounded-lg bg-card">
              <div className="text-2xl font-bold text-green-600">
                {syncStatus.in_sync_count}
              </div>
              <div className="text-sm text-muted-foreground">In Sync</div>
            </div>
            <div className="p-4 border rounded-lg bg-card">
              <div className="text-2xl font-bold text-blue-600">
                {syncStatus.needs_upload_count}
              </div>
              <div className="text-sm text-muted-foreground">Needs Upload</div>
            </div>
            <div className="p-4 border rounded-lg bg-card">
              <div className="text-2xl font-bold text-blue-600">
                {syncStatus.needs_download_count}
              </div>
              <div className="text-sm text-muted-foreground">Needs Download</div>
            </div>
            <div className="p-4 border rounded-lg bg-card">
              <div className="text-2xl font-bold text-red-600">
                {syncStatus.conflict_count}
              </div>
              <div className="text-sm text-muted-foreground">Conflicts</div>
            </div>
          </div>
        )}

        {/* File list */}
        {syncStatus && syncStatus.comparisons.length > 0 ? (
          <FileList files={syncStatus.comparisons} />
        ) : (
          <div className="p-12 border rounded-lg bg-card text-center text-muted-foreground">
            {localPath
              ? isLoading
                ? "Loading files..."
                : "No files found in the selected folder"
              : "Select a folder to view files"}
          </div>
        )}
      </div>
    </div>
  );
}
