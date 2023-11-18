import { Button, Textarea } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { rspc } from "../lib/client";
import { useRef } from "react";

export function ConfigTab() {
  const { data: config } = rspc.useQuery(["config_read"]);
  const { mutateAsync: writeConfig } = rspc.useMutation(["config_write"]);
  const ref = useRef<HTMLTextAreaElement>(null);

  return (
    <div className="flex flex-col gap-2 p-2">
      {config && (
        <>
          <Textarea
            classNames={{
              input: "font-mono",
            }}
            ref={ref}
            defaultValue={config}
            autosize
            variant="filled"
            spellCheck={false}
          />
          <Button
            onClick={async () => {
              if (ref.current?.value) {
                await writeConfig(ref.current?.value);
                notifications.show({
                  title: "Success",
                  message: "Saved config",
                });
              }
            }}
          >
            Save
          </Button>
        </>
      )}
    </div>
  );
}
