import { rspc } from "../lib/client";
import { useEffect, useState } from "react";
import { LogTable } from "./LogTable";
import { ScanLog } from "../lib/bindings";
import { LogView } from "./LogView";
import { ScanForm } from "./ScanForm";
import { Table } from "@mantine/core";

export const perPage = 10;

function App() {
  const [logPage, setLogPage] = useState(0);
  const { data: log } = rspc.useQuery(["scan_log", {
    limit: perPage,
    page: logPage,
  }], {
    refetchInterval: () => {
      if (logPage == 0) return 2000;
      return false;
    },
  });

  const { data: queueInfo } = rspc.useQuery(["queue_info"], {
    refetchInterval: 3000,
  });
  useEffect(() => {
    console.log("queueInfo", queueInfo);
  }, [queueInfo]);
  const [logToShow, setLogToShow] = useState<ScanLog | null>(null);
  return (
    <div className="flex justify-center p-3 h-full">
      <div className="w-full flex flex-col gap-2">
        <h1 className="text-2xl">
          Tagbrain
        </h1>
        <div className="grid grid-rows-2 lg:grid-rows-none lg:grid-cols-2 gap-2">
          <ScanForm />
          <div>
            {queueInfo && (
              <Table
                data={{
                  body: [
                    ["queue length", queueInfo.queue_count],
                    [
                      "current job",
                      JSON.stringify(queueInfo.current_job, null, 2),
                    ],
                  ],
                }}
              />
            )}
          </div>
        </div>
        <div className="lg:grid lg:grid-cols-5 gap-2">
          {log && (
            <LogTable
              className="lg:col-span-3 whitespace-nowrap"
              data={log[0]}
              changePage={(page) => setLogPage(page - 1)}
              page={logPage + 1}
              maxPage={Math.ceil(log[1] / perPage)}
              onRowClick={(row) => setLogToShow(row)}
              totalRecords={log[1]}
              recordsPerPage={perPage}
            />
          )}
          {logToShow && (
            <div className="lg:col-span-2">
              <LogView log={logToShow} />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
