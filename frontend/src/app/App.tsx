import { Tabs } from "@mantine/core";
import { DownloaderTab } from "./DownloaderTab";
import { ConfigTab } from "./ConfigTab";

function App() {
  return (
    <Tabs defaultValue="downloader">
      <Tabs.List>
        <Tabs.Tab value="downloader">
          Downloader
        </Tabs.Tab>
        <Tabs.Tab value="config">
          Config
        </Tabs.Tab>
      </Tabs.List>

      <Tabs.Panel value="downloader">
        <DownloaderTab />
      </Tabs.Panel>

      <Tabs.Panel value="config">
        <ConfigTab />
      </Tabs.Panel>
    </Tabs>
  );
}

export default App;
