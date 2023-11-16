import { DataTable } from "mantine-datatable";
import { ScanLog } from "../lib/bindings";
import { useState } from "react";

export function LogTable(
  props: {
    data: ScanLog[];
    changePage: (page: number) => void;
    page: number;
    maxPage: number;
    recordsPerPage: number;
    onRowClick: (row: ScanLog) => void;
    totalRecords: number;
    className?: string;
  },
) {
  const [selectedRow, setSelectedRow] = useState<number | null>(null);
  return (
    <div
      className={props.className}
    >
      <DataTable
        height="auto"
        records={props.data}
        withTableBorder
        rowClassName={(_data, i) => {
          if (i == selectedRow) return "bg-gray-300/30";
        }}
        columns={[
          { accessor: "id", title: "#", textAlign: "right" },
          {
            accessor: "created_at",
            title: "date",
            render: ({ created_at }) =>
              new Date(created_at * 1000).toLocaleString(),
          },
          {
            accessor: "success",
            render: ({ success }) =>
              success
                ? <span className="text-blue-600">TRUE</span>
                : <span className="text-red-600">FALSE</span>,
          },
          { accessor: "source_path", title: "path" },
          {
            accessor: "acoustid_score",
            title: "match",
            render: ({ acoustid_score }) =>
              acoustid_score ? Math.round(acoustid_score * 100) + "%" : "-",
          },
        ]}
        page={props.page}
        onPageChange={props.changePage}
        totalRecords={props.totalRecords}
        recordsPerPage={props.recordsPerPage}
        onRowClick={({ record, index }) => {
          setSelectedRow(index);
          props.onRowClick(record);
        }}
      />
    </div>
  );
}
