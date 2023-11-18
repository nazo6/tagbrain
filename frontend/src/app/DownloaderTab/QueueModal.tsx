import { DataTable } from "mantine-datatable";
import { JobTask } from "../../lib/bindings";

export function QueueModal(
  props: {
    tasks: JobTask[];
  },
) {
  const tasks = props.tasks.map((t, i) => {
    if ("Scan" in t) {
      return {
        id: i + 1,
        type: "Scan",
        message: `path:${t.Scan.path}, retry_count:${t.Scan.retry_count}`,
      };
    } else {
      return {
        id: i + 1,
        type: "Fix",
        message: `path:${t.Fix.path}, copy:${t.Fix.copy_to_target}`,
      };
    }
  });
  return (
    <div className="flex flex-col gap-2">
      <span>Queue count: {tasks.length}</span>
      <DataTable
        height="470px"
        records={tasks}
        withTableBorder
        columns={[
          { accessor: "id" },
          { accessor: "path" },
          { accessor: "retry_count" },
        ]}
      />
    </div>
  );
}
