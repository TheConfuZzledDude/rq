import { invoke } from "@tauri-apps/api";
import { appWindow, PhysicalSize } from "@tauri-apps/api/window";
import { useEffect, useRef } from "preact/hooks";

import "@/css/app.css";
import "@/css/themes/98/98.scss";
import useSize from "@react-hook/size";

export const NewQueue = () => {
  const queueNameEl = useRef<HTMLInputElement>(null);
  const restrictToGroupEl = useRef<HTMLInputElement>(null);

  const mainContentRef = useRef<HTMLDivElement>(null);
  const [width, height] = useSize(mainContentRef);

  useEffect(() => {
    (async () => {
      appWindow.setSize(
        new PhysicalSize((await appWindow.innerSize()).width, height * (await appWindow.scaleFactor())),
      );
    })();
  }, [height, width]);

  return (
    <div ref={mainContentRef} id="app">
      <div data-tauri-drag-region={true} id="titlebar" class="titlebar">
        <div class="titlebar-logo-container">
          <img class="titlebar-logo" />
        </div>
        <div class="titlebar-button" id="titlebar-minimize" onClick={appWindow.minimize} alt="minimize" />
        <div class="titlebar-button" id="titlebar-close" alt="close" onClick={appWindow.close} />
      </div>
      <div class="new-queue-container">
        <label for="name">Queue Name</label>
        <input ref={queueNameEl} id="name" type="text" class="new-queue-name" />
        <label for="restrict-group">Restrict To Group</label>
        <input ref={restrictToGroupEl} id="restrict-group" type="text" class="new-queue-restrict-group" />
        <input
          type="submit"
          onClick={() => {
            queueNameEl.current &&
              restrictToGroupEl.current &&
              newQueue(queueNameEl.current.value.trim(), restrictToGroupEl.current.value.trim() || "");
            appWindow.close();
          }}
        />
      </div>
    </div>
  );
};

const newQueue = (name: string, restrictToGroup?: string) => {
  invoke("new_queue", { name, restrictToGroup });
};
