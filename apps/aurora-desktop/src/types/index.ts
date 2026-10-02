export interface SegmentDto {
  index: number;
  start_byte: number;
  end_byte: number;
  downloaded_bytes: number;
  state: 'pending' | 'downloading' | 'completed' | 'failed';
  speed_bps: number;
  stream_id?: number;
}

export interface DownloadItemDto {
  id: string;
  url: string;
  filename: string;
  destination_path: string;
  total_bytes: number | null;
  downloaded_bytes: number;
  current_speed: number;
  peak_speed: number;
  min_speed: number;
  eta_secs: number | null;
  status: 'downloading' | 'completed' | 'paused' | 'failed' | 'cancelled' | string;
  active_connections: number;
  http_version: string;
  etag: string | null;
  accepts_ranges: boolean;
  is_active: boolean;
  segments: SegmentDto[];
  start_time_str: string;
  end_time_str: string | null;
  elapsed_duration_secs: number | null;
  server_rtt_ms: number | null;
  server_name: string | null;
  health_rating: string | null;
  checksum_result: string | null;
}

export interface ServerCapabilitiesDto {
  url: string;
  content_length: number | null;
  accepts_ranges: boolean;
  etag: string | null;
  content_type: string | null;
  server_name: string | null;
  http_version: string;
  rtt_ms: number | null;
  health_rating: string | null;
  suggested_filename: string | null;
}

export interface EngineConfigDto {
  download_dir: string;
  initial_connections: number;
  max_connections: number;
  chunk_size_bytes: number;
  write_buffer_bytes: number;
  request_timeout_secs: number;
  max_retries: number;
}

export interface TelemetryPayload {
  total_speed_bps: number;
  total_speed_str: string;
  active_tasks: number;
  speed_history: number[];
  downloads: DownloadItemDto[];
}

export type CategoryFilter =
  | 'all'
  | 'active'
  | 'completed'
  | 'paused'
  | 'failed'
  | 'video'
  | 'audio'
  | 'documents'
  | 'compressed'
  | 'applications';

export type SortColumn = 'name' | 'size' | 'status' | 'speed' | 'time' | 'eta';
export type SortDirection = 'asc' | 'desc';
