import { useState } from "react";

import { Box, Button, Checkbox, Group, TextInput } from "@mantine/core";
import { useForm } from "@mantine/form";
import { client } from "./client";
import { notifications } from "@mantine/notifications";

function App() {
  const form = useForm({
    initialValues: {
      scanPath: "",
    },
  });

  return (
    <div className="flex flex-col gap-2">
      <div className="flex gap-2 items-center">
        <TextInput
          placeholder="Scan path"
          {...form.getInputProps("scanPath")}
        />
        <Button
          onClick={async () => {
            try {
              await client.api.scan({ path: form.values.scanPath });
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
              await client.api.scanAll();
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

export default App;
