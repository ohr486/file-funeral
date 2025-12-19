import { useState } from "react";
import { RefreshCw, CheckCircle2, AlertCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { toast } from "sonner";
import { invoke } from "@tauri-apps/api/core";
import { SyncFilesResponse } from "@/types";

interface SyncButtonProps {
  localPath: string;
  remotePrefix: string;
  onSyncComplete?: () => void;
  disabled?: boolean;
}

type SyncStatus = "idle" | "syncing" | "success" | "error";

export function SyncButton({
  localPath,
  remotePrefix,
  onSyncComplete,
  disabled = false,
}: SyncButtonProps) {
  const [status, setStatus] = useState<SyncStatus>("idle");
  const [progress, setProgress] = useState(0);
  const [message, setMessage] = useState("");

  const handleSync = async () => {
    if (!localPath) {
      toast.error("Please select a local folder first");
      return;
    }

    setStatus("syncing");
    setProgress(0);
    setMessage("Starting synchronization...");

    try {
      // Simulate progress updates
      const progressInterval = setInterval(() => {
        setProgress((prev) => {
          if (prev >= 90) {
            clearInterval(progressInterval);
            return prev;
          }
          return prev + 10;
        });
      }, 200);

      const response = await invoke<SyncFilesResponse>("sync_files", {
        request: {
          local_path: localPath,
          remote_prefix: remotePrefix,
        },
      });

      clearInterval(progressInterval);
      setProgress(100);

      if (response.success) {
        setStatus("success");
        setMessage(
          `Sync completed: ${response.files_uploaded} uploaded, ${response.files_downloaded} downloaded, ${response.conflicts_resolved} conflicts resolved`
        );
        toast.success("Synchronization completed successfully", {
          description: `${response.files_uploaded} files uploaded, ${response.files_downloaded} files downloaded`,
        });

        if (onSyncComplete) {
          onSyncComplete();
        }

        // Reset status after a delay
        setTimeout(() => {
          setStatus("idle");
          setProgress(0);
          setMessage("");
        }, 3000);
      } else {
        setStatus("error");
        setMessage(response.message);
        toast.error("Synchronization failed", {
          description: response.message,
        });

        setTimeout(() => {
          setStatus("idle");
          setProgress(0);
          setMessage("");
        }, 3000);
      }
    } catch (error) {
      setStatus("error");
      const errorMessage = error instanceof Error ? error.message : String(error);
      setMessage(errorMessage);
      toast.error("Synchronization error", {
        description: errorMessage,
      });

      setTimeout(() => {
        setStatus("idle");
        setProgress(0);
        setMessage("");
      }, 3000);
    }
  };

  const getButtonContent = () => {
    switch (status) {
      case "syncing":
        return (
          <>
            <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
            Syncing...
          </>
        );
      case "success":
        return (
          <>
            <CheckCircle2 className="mr-2 h-4 w-4" />
            Sync Complete
          </>
        );
      case "error":
        return (
          <>
            <AlertCircle className="mr-2 h-4 w-4" />
            Sync Failed
          </>
        );
      default:
        return (
          <>
            <RefreshCw className="mr-2 h-4 w-4" />
            Sync Files
          </>
        );
    }
  };

  const getButtonVariant = () => {
    switch (status) {
      case "success":
        return "default";
      case "error":
        return "destructive";
      default:
        return "default";
    }
  };

  return (
    <div className="space-y-3">
      <Button
        onClick={handleSync}
        disabled={disabled || status === "syncing"}
        variant={getButtonVariant()}
        className="w-full"
      >
        {getButtonContent()}
      </Button>

      {status === "syncing" && (
        <div className="space-y-2">
          <Progress value={progress} className="w-full" />
          <p className="text-sm text-muted-foreground text-center">{message}</p>
        </div>
      )}

      {(status === "success" || status === "error") && message && (
        <p
          className={`text-sm text-center ${
            status === "success" ? "text-green-600" : "text-red-600"
          }`}
        >
          {message}
        </p>
      )}
    </div>
  );
}
