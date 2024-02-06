import { rspc } from "../../lib/client";
import { useEffect, useState } from "react";
import { LogTable } from "./LogTable";
import { QueueInfo as QueueInfoType, ScanLog } from "../../lib/bindings";
import { LogView } from "./LogView";
import { ScanForm } from "./ScanForm";
import { Button, Checkbox, Modal, Table } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { QueueModal } from "./QueueModal";

export const perPage = 10;

export function MainTab() {
  const [logPage, setLogPage] = useState(0);
  const [failedOnly, setFailedOnly] = useState(false);

  const { data: log } = rspc.useQuery(["scan_log", {
    limit: perPage,
    page: logPage,
    success: failedOnly ? false : null,
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
            {queueInfo && <QueueInfo queueInfo={queueInfo} />}
          </div>
        </div>
        <div className="lg:grid lg:grid-cols-5 gap-2">
          {log && (
            <div className="flex flex-col lg:col-span-3 whitespace-nowrap gap-2">
              <Checkbox
                label="Failed only"
                checked={failedOnly}
                onChange={(e) => setFailedOnly(e.currentTarget.checked)}
              />
              <LogTable
                data={log[0]}
                changePage={(page) => setLogPage(page - 1)}
                page={logPage + 1}
                maxPage={Math.ceil(log[1] / perPage)}
                onRowClick={(row) => setLogToShow(row)}
                totalRecords={log[1]}
                recordsPerPage={perPage}
              />
            </div>
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

function QueueInfo(props: { queueInfo: QueueInfoType }) {
  const [queueInfoOpened, { open: openQueueInfo, close: closeQueueInfo }] =
    useDisclosure(false);
  const tasks = props.queueInfo.tasks.slice().reverse();
  return (
    <div>
      <Table
        className="whitespace-pre-wrap"
        data={{
          body: [
            ["queue length", tasks.length],
            [
              "current job",
              tasks[0]
                ? "Scan" in tasks[0] ? tasks[0].Scan.path : tasks[0].Fix.path
                : null,
            ],
          ],
        }}
      />
      <Button onClick={openQueueInfo}>Queue details</Button>

      <Modal
        size="auto"
        opened={queueInfoOpened}
        onClose={closeQueueInfo}
        title="Queue Info"
      >
        <QueueModal tasks={tasks} />
      </Modal>
    </div>
  );
}
