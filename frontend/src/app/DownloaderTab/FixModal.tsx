import { ScanLog } from "../../lib/bindings";
import { Button, TextInput } from "@mantine/core";
import { useForm } from "@mantine/form";
import { rspc } from "../../lib/client";
import { notifications } from "@mantine/notifications";
import { modals } from "@mantine/modals";

export function FixModal(props: { scanLog: ScanLog }) {
  const form = useForm({
    initialValues: {
      recordingId: "",
      releaseId: "",
    },
  });

  const { mutateAsync: fix } = rspc.useMutation("fix");
  const { mutateAsync: fixFailed } = rspc.useMutation("fix_failed");

  return (
    <>
      <TextInput
        label="Musicbrainz release id"
        {...form.getInputProps("releaseId")}
      />
      <TextInput
        label="Musicbrainz recording id"
        {...form.getInputProps("recordingId")}
      />
      <Button
        fullWidth
        onClick={async () => {
          if (props.scanLog.success) {
            if (props.scanLog.target_path) {
              await fix({
                target_path: props.scanLog.target_path,
                recording_id: form.values.recordingId,
                release_id: form.values.releaseId,
              });
            } else {
              throw new Error("target_path is null");
            }
          } else {
            await fixFailed({
              source_path: props.scanLog.source_path,
              recording_id: form.values.recordingId,
              release_id: form.values.releaseId,
            });
          }
          notifications.show({
            title: "Success",
            message: "Send request",
          });
          modals.closeAll();
        }}
        mt="md"
      >
        Submit
      </Button>
    </>
  );
}
