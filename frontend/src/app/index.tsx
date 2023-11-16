import App from "./App.tsx";
import { createTheme, MantineProvider } from "@mantine/core";
import { Notifications } from "@mantine/notifications";
import { QueryClient } from "@tanstack/react-query";
import { createClient, WebsocketTransport } from "@rspc/client";
import { Procedures } from "../lib/bindings.ts";
import { rspc } from "../lib/client.ts";

import "@mantine/core/styles.css";
import "@mantine/notifications/styles.css";
import "mantine-datatable/styles.css";
import "./index.css";

let baseUrl = "ws://localhost:3080";
if (import.meta.env.PROD) {
  baseUrl = "wss://" + location.host;
}

const client = createClient<Procedures>({
  transport: new WebsocketTransport(baseUrl + "/rspc/ws"),
});

const queryClient = new QueryClient();

const theme = createTheme({});
export function AppIndex() {
  return (
    <rspc.Provider client={client} queryClient={queryClient}>
      <MantineProvider theme={theme}>
        <Notifications />
        <App />
      </MantineProvider>
    </rspc.Provider>
  );
}
