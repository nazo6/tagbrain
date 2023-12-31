import { ScanLog } from "../../lib/bindings";
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
                  ["target", log.target_path],
                  ["message", log.message],
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
                  getRow(log, "disc"),
                  getRow(log, "total_discs"),
                  getRow(log, "date"),
                  getRow(log, "year"),
                  getRow(log, "label"),
                  getRow(log, "media"),
                  getRow(log, "musicbrainz_track_id"),
                  getRow(log, "musicbrainz_release_id"),
                  getRow(log, "musicbrainz_artist_id"),
                  getRow(log, "musicbrainz_release_artist_id"),
                  getRow(log, "musicbrainz_release_group_id"),
                  getRow(log, "musicbrainz_recording_id"),
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
