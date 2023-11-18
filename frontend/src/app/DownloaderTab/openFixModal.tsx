import { modals } from "@mantine/modals";
import { ScanLog } from "../../lib/bindings";
import { FixModal } from "./FixModal";

export function openFixModal(scanLog: ScanLog) {
  modals.open({
    title: "Fix metadata",
    children: <FixModal scanLog={scanLog} />,
  });
}
