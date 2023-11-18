// This file was generated by [rspc](https://github.com/oscartbeaumont/rspc). Do not edit this file manually.

export type Procedures = {
    queries: 
        { key: "config_read", input: never, result: string } | 
        { key: "queue_info", input: never, result: QueueInfo } | 
        { key: "scan_log", input: ScanLogRequest, result: [ScanLog[], number] },
    mutations: 
        { key: "config_write", input: string, result: null } | 
        { key: "fix", input: FixRequest, result: null } | 
        { key: "fix_failed", input: FixFailedRequest, result: null } | 
        { key: "queue_clear", input: never, result: null } | 
        { key: "scan", input: ScanRequest, result: null } | 
        { key: "scan_all", input: never, result: null },
    subscriptions: never
};

export type FixFailedRequest = { source_path: string; release_id: string; recording_id: string }

export type FixRequest = { target_path: string; release_id: string; recording_id: string }

export type ScanLogRequest = { limit: number; page: number }

export type LogType = "Scan" | "Fix"

export type QueueInfo = { tasks: JobTask[] }

export type ScanRequest = { path: string }

export type Metadata = { title: string | null; artist: string | null; artist_sort: string | null; album: string | null; album_artist: string | null; album_artist_sort: string | null; track: string | null; total_tracks: string | null; disk: string | null; total_disks: string | null; original_date: string | null; date: string | null; year: string | null; label: string | null; media: string | null; script: string | null; musicbrainz_track_id: string | null; musicbrainz_recording_id: string | null; musicbrainz_artist_id: string | null; musicbrainz_release_id: string | null; musicbrainz_release_artist_id: string | null; musicbrainz_release_group_id: string | null }

export type ScanLog = { id: number; type: LogType; created_at: number; success: boolean; message: string | null; old_metadata: Metadata | null; new_metadata: Metadata | null; source_path: string; target_path: string | null; acoustid_score: number | null; retry_count: number | null }

export type JobTask = { Scan: { path: string; retry_count: number } } | { Fix: { path: string; release_id: string; recording_id: string; copy_to_target: boolean } }
