// This file was generated by [rspc](https://github.com/oscartbeaumont/rspc). Do not edit this file manually.

export type Procedures = {
    queries: 
        { key: "queue_info", input: never, result: QueueInfo } | 
        { key: "scan_log", input: ScanLogRequest, result: [ScanLog[], number] },
    mutations: 
        { key: "scan", input: ScanRequest, result: null } | 
        { key: "scan_all", input: never, result: null },
    subscriptions: never
};

export type ScanLog = { id: number; created_at: number; success: boolean; message: string | null; old_metadata: Metadata | null; new_metadata: Metadata | null; source_path: string; target_path: string | null; acoustid_score: number | null; retry_count: number }

export type ScanRequest = { path: string }

export type JobTask = { Scan: { path: string; retry_count: number } }

export type Metadata = { title: string | null; artist: string | null; artist_sort: string | null; album: string | null; album_artist: string | null; album_artist_sort: string | null; track: string | null; total_tracks: string | null; disk: string | null; total_disks: string | null; date: string | null; year: string | null; label: string | null; media: string | null; musicbrainz_track_id: string | null; musicbrainz_album_id: string | null; musicbrainz_artist_id: string | null; musicbrainz_release_artist_id: string | null; musicbrainz_release_group_id: string | null }

export type ScanLogRequest = { limit: number; page: number }

export type QueueInfo = { queue_count: number; current_job: JobTask | null }
