import { DataTable } from "mantine-datatable";
import { JobTask } from "../../lib/bindings";

export function QueueModal(
  props: {
    tasks: JobTask[];
  },
) {
  const tasks = props.tasks.map((t, i) => {
    return {
      id: i + 1,
      ...t.Scan,
    };
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
