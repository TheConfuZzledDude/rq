import { useCallback, useEffect, useRef, useState } from "preact/hooks";
import { listen, TauriEvent } from "@tauri-apps/api/event";
import { QueueCard } from "@/components/queuecard";
import { invoke } from "@tauri-apps/api/tauri";
import { Queue } from "@/data/queue";
import { User } from "@/data/user";
import useSize from "@react-hook/size";
import { appWindow, currentMonitor, PhysicalSize } from "@tauri-apps/api/window";

import rqLogo from "@/../icons/128x128.png";

import "@/css/app.css";
import "@szhsin/react-menu/dist/core.css";

// import "@/css/themes/default_theme.scss";
// import "@/css/themes/98/98.scss";

import { ClickEvent, ControlledMenu, MenuItem, useMenuState } from "@szhsin/react-menu";
import { exit } from "@tauri-apps/api/process";

import { Settings, Theme } from "@/data/settings";

import { themeCssMap } from "@/utilities";
import { event } from "@tauri-apps/api";

interface PollDataResponse {
  queues: [Queue];
  config: Settings;
}

export const App = () => {
  const [queues, setQueues] = useState<Record<number, Queue>>();

  const [currentUser, setCurrentUser] = useState<User>({
    username: "",
    email: "",
    fullName: "",
  });
  const [theme, setTheme] = useState<Theme>("Win98");
  const [hiddenQueues, setHiddenQueues] = useState(new Set());

  const hideQueue = useCallback(
    (id: number) => {
      const newSet = new Set(hiddenQueues);
      newSet.add(id);
      setHiddenQueues(newSet);
    },
    [hiddenQueues],
  );

  const appRef = useRef<HTMLDivElement>(null);
  const [width, height] = useSize(appRef);

  const titlebarRef = useRef<HTMLDivElement>(null);
  const [, titlebarHeight] = useSize(titlebarRef);

  const queueContainerRef = useRef<HTMLDivElement>(null);

  const [menuProps, toggleMenu] = useMenuState();
  const contextMenuRef = useRef(null);

  const [maxHeight, setMaxHeight] = useState(0);

  useEffect(() => {
    (async () => {
      const scaleFactor = await appWindow.scaleFactor();

      appWindow.setSize(
        new PhysicalSize(
          (await appWindow.innerSize()).width,

          Math.max(height * scaleFactor, 50),
        ),
      );
    })();
  }, [height, width, titlebarHeight, maxHeight]);

  useEffect(() => {
    let unlistenQueuesUpdated: (() => void) | null = null;
    let unlistenWindowMove: (() => void) | null = null;
    // let unlistenMousePosition: (() => void) | null = null;
    async function fetchData() {
      unlistenQueuesUpdated = await listen<PollDataResponse>("data_updated", ({ payload: { config, queues } }) => {
        setQueues(queues);
        setCurrentUser({
          email: config.email,
          username: config.username,
          fullName: config.fullName,
        });
        setTheme(config.theme);
      });
      unlistenWindowMove = await listen<event.Event<TauriEvent.WINDOW_MOVED>>(TauriEvent.WINDOW_MOVED, async (e) => {
        console.log("Window moved");
        const monitor = await currentMonitor();
        const scaleFactor = await appWindow.scaleFactor();
        setMaxHeight(((monitor?.size.height ?? 0) * 0.9) / scaleFactor);
      });

      await invoke("fetch_data");
    }
    fetchData();
    return () => {
      unlistenQueuesUpdated?.();
      unlistenWindowMove?.();
      // unlistenMousePosition && unlistenMousePosition();
    };
  }, []);

  return (
    <>
      <style>{themeCssMap[theme] ?? ""}</style>
      <div style={{ maxHeight: maxHeight }} ref={appRef} id="app">
        <div ref={titlebarRef} data-tauri-drag-region={true} id="titlebar" class="titlebar">
          <div class="titlebar-logo-container">
            <img class="titlebar-logo" src={rqLogo} />
          </div>
          <div
            class={`titlebar-button ${menuProps.state === "open" ? "active" : ""}`}
            id="titlebar-menu"
            ref={contextMenuRef}
            onClick={() => toggleMenu(true)}
            alt="menu"
          />
          <div class="titlebar-button" id="titlebar-minimize" onClick={appWindow.minimize} alt="minimize" />
          <div class="titlebar-button" id="titlebar-maximize" onClick={appWindow.maximize} alt="maximize" />
          <div class="titlebar-button" id="titlebar-close" alt="close" onClick={appWindow.hide} />
        </div>
        <ControlledMenu
          {...menuProps}
          anchorRef={contextMenuRef}
          menuClassName="context-menu"
          onMouseLeave={() => toggleMenu(false)}
          onClose={() => toggleMenu(false)}
          onItemClick={({ value }: ClickEvent) => {
            switch (value) {
              case "new": {
                invoke("open_new_queue");
                break;
              }
              case "restore": {
                setHiddenQueues(new Set());
                break;
              }
              case "settings": {
                invoke("open_settings");
                break;
              }
              case "exit": {
                exit(0);
              }
            }
          }}
        >
          <MenuItem value="new">New Queue</MenuItem>
          <MenuItem value="restore">Restore Hidden Items</MenuItem>
          <MenuItem value="settings">Settings</MenuItem>
        </ControlledMenu>
        <div class="shrink-wrapper" style={{ overflowY: "auto" }}>
          <div ref={queueContainerRef} class="queues-container">
            {
              Object.entries(queues ?? {})
              ?.filter(([, queue]) => !hiddenQueues.has(queue.id))
              .map(([, queue]) => (
                <QueueCard key={queue.id} queue={queue} user={currentUser} onHide={hideQueue} />
              ))}
          </div>
        </div>
      </div>
    </>
  );
};
