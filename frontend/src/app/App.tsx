import { Tabs } from "@mantine/core";
import { MainTab } from "./MainTab";
import { ConfigTab } from "./ConfigTab";

function App() {
  return (
    <Tabs defaultValue="main">
      <Tabs.List>
        <Tabs.Tab value="main">
          Main
        </Tabs.Tab>
        <Tabs.Tab value="config">
          Config
        </Tabs.Tab>
      </Tabs.List>

      <Tabs.Panel value="main">
        <MainTab />
      </Tabs.Panel>

      <Tabs.Panel value="config">
        <ConfigTab />
      </Tabs.Panel>
    </Tabs>
  );
}

export default App;
