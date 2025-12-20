// Type definitions for Tauri commands and responses

export enum SyncState {
  InSync = "InSync",
  NeedsUpload = "NeedsUpload",
  NeedsDownload = "NeedsDownload",
  Conflict = "Conflict",
}

export interface FileInfo {
  path: string;
  size: number;
  last_modified: string;
  etag?: string;
}

export interface ComparisonResultDto {
  path: string;
  state: SyncState;
  local_size?: number;
  remote_size?: number;
  local_modified?: string;
  remote_modified?: string;
}

export interface SyncStatusResponse {
  comparisons: ComparisonResultDto[];
  in_sync_count: number;
  needs_upload_count: number;
  needs_download_count: number;
  conflict_count: number;
}

export interface SyncFilesResponse {
  success: boolean;
  files_uploaded: number;
  files_downloaded: number;
  conflicts_resolved: number;
  message: string;
}

export interface ListFilesResponse {
  files: FileInfo[];
  total_count: number;
}

export interface SetCredentialsRequest {
  access_key_id: string;
  secret_access_key: string;
  region: string;
  bucket_name: string;
}

export interface CredentialsResponse {
  success: boolean;
  message: string;
}

export interface GetCredentialsResponse {
  has_credentials: boolean;
  access_key_id?: string;
  region?: string;
  bucket_name?: string;
}

export interface ConnectionTestResponse {
  connected: boolean;
  message: string;
  region?: string;
  bucket_name?: string;
}
