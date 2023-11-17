import { ScanLog } from "../lib/bindings";
import { Table } from "@mantine/core";

function getRow(log: ScanLog, prop: string) {
  return [
    prop,
    // @ts-expect-error aaa
    log.old_metadata ? log.old_metadata[prop] : null,
    // @ts-expect-error aaa
    log.new_metadata ? log.new_metadata[prop] : null,
  ];
}

export type Metadata = {
  title: string | null;
  artist: string | null;
  artist_sort: string | null;
  album: string | null;
  album_artist: string | null;
  album_artist_sort: string | null;
  track: string | null;
  total_tracks: string | null;
  disk: string | null;
  total_disks: string | null;
  date: string | null;
  year: string | null;
  label: string | null;
  media: string | null;
  musicbrainz_track_id: string | null;
  musicbrainz_release_id: string | null;
  musicbrainz_artist_id: string | null;
  musicbrainz_release_artist_id: string | null;
  musicbrainz_release_group_id: string | null;
};

export function LogView({ log }: { log: ScanLog }) {
  return (
    <div className="flex flex-col">
      <h2 className="text-xl">Details of #{log.id}</h2>
      {log.success
        ? (
          <div className="flex flex-col">
            <Table
              data={{
                head: [
                  "title",
                  "",
                ],
                body: [
                  ["moved to", log.target_path],
                  ["retry times", log.retry_count],
                ],
              }}
            />
            <h3 className="text-lg">Metadata</h3>
            <Table
              data={{
                head: [
                  "title",
                  "old",
                  "new",
                ],
                body: [
                  getRow(log, "title"),
                  getRow(log, "artist"),
                  getRow(log, "artist_sort"),
                  getRow(log, "album"),
                  getRow(log, "album_artist"),
                  getRow(log, "album_artist_sort"),
                  getRow(log, "track"),
                  getRow(log, "total_tracks"),
                  getRow(log, "disk"),
                  getRow(log, "total_disks"),
                  getRow(log, "date"),
                  getRow(log, "year"),
                  getRow(log, "label"),
                  getRow(log, "media"),
                  getRow(log, "musicbrainz_track_id"),
                  getRow(log, "musicbrainz_release_id"),
                  getRow(log, "musicbrainz_artist_id"),
                  getRow(log, "musicbrainz_release_artist_id"),
                  getRow(log, "musicbrainz_release_group_id"),
                ],
              }}
            />
          </div>
        )
        : (
          <div className="flex flex-col whitespace-pre-wrap">
            <Table
              data={{
                head: [
                  "title",
                  "",
                ],
                body: [
                  ["error message", log.message],
                  ["retry times", log.retry_count],
                ],
              }}
            />
          </div>
        )}
    </div>
  );
}
