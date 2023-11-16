import { Button, TextInput } from "@mantine/core";
import { useForm } from "@mantine/form";
import { rspc } from "../lib/client";
import { notifications } from "@mantine/notifications";

export function ScanForm() {
  const form = useForm({
    initialValues: {
      scanPath: "",
    },
  });

  const { mutateAsync: scan } = rspc.useMutation("scan");
  const { mutateAsync: scanAll } = rspc.useMutation("scan_all");

  return (
    <div>
      <div className="flex gap-2 items-center">
        <TextInput
          placeholder="Scan path"
          className="flex-grow"
          {...form.getInputProps("scanPath")}
        />
        <Button
          onClick={async () => {
            try {
              await scan({ path: form.values.scanPath });
              notifications.show({
                title: "Success",
                message: "Send request",
              });
            } catch (e: any) {
              notifications.show({
                title: "Error",
                message: e.message ?? "Unknown error",
              });
            }
          }}
        >
          Scan
        </Button>
      </div>
      <div className="flex">
        <Button
          onClick={async () => {
            try {
              await scanAll(undefined);
              notifications.show({
                title: "Success",
                message: "Send request",
              });
            } catch (e: any) {
              notifications.show({
                title: "Error",
                message: e.message ?? "Unknown error",
              });
            }
          }}
        >
          Scan all files
        </Button>
      </div>
    </div>
  );
}
