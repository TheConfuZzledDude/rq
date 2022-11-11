import { invoke } from "@tauri-apps/api";
import { appWindow, PhysicalSize } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "preact/hooks";

import "@/css/app.css";
import "@/css/themes/98/98.scss";
import useSize from "@react-hook/size";

import { Settings, themes } from "@/data/settings";

const SettingsComponent = () => {
  const fullNameEl = useRef<HTMLInputElement>(null);
  const usernameEl = useRef<HTMLInputElement>(null);
  const emailEl = useRef<HTMLInputElement>(null);
  const groupsEl = useRef<HTMLInputElement>(null);
  const themeEl = useRef<HTMLSelectElement>(null);

  const mainContentRef = useRef<HTMLDivElement>(null);
  const [width, height] = useSize(mainContentRef);

  const [settings, setSettings] = useState<Settings | undefined>();

  useEffect(() => {
    (async () => {
      appWindow.setSize(
        new PhysicalSize((await appWindow.innerSize()).width, height * (await appWindow.scaleFactor())),
      );
    })();
  }, [height, width]);

  useEffect(() => {
    invoke<Settings>("fetch_settings").then(setSettings);
  }, []);

  return (
    <div ref={mainContentRef} id="app">
      <div data-tauri-drag-region={true} id="titlebar" class="titlebar">
        <div class="titlebar-logo-container">
          <img class="titlebar-logo" />
        </div>
        <div class="titlebar-button" id="titlebar-minimize" onClick={appWindow.minimize} alt="minimize" />
        <div class="titlebar-button" id="titlebar-close" alt="close" onClick={appWindow.close} />
      </div>
      <div class="settings-container">
        <label for="full_name">Full Name</label>
        <input ref={fullNameEl} id="full_name" type="text" class="settings-full-name" value={settings?.["fullName"]} />
        <label for="username">Username</label>
        <input ref={usernameEl} id="username" type="text" class="settings-username" value={settings?.["username"]} />
        <label for="email">Email</label>
        <input ref={emailEl} id="email" type="text" class="settings-email" value={settings?.["email"]} />
        <label for="groups">Groups (Comma Separated)</label>
        <input ref={groupsEl} id="groups" type="text" class="settings-groups" value={settings?.["groups"]?.join(",")} />
        <label for="theme">Theme</label>
        <select ref={themeEl} id="theme" class="settings-themes" value={settings?.["theme"] ?? "Modern"}>
          {themes.map((theme) => (
            <option key={theme} value={theme}>
              {theme}
            </option>
          ))}
        </select>
        <input
          type="submit"
          onClick={() => {
            invoke("write_settings", {
              settings: {
                fullName: fullNameEl.current?.value ?? "",
                username: usernameEl.current?.value ?? "",
                email: emailEl.current?.value ?? "",
                groups: (groupsEl.current?.value ?? "").trim().split(","),
                theme: themeEl.current?.value ?? "Modern",
              },
            });

            console.log({
              fullName: fullNameEl.current?.value ?? "",
              username: usernameEl.current?.value ?? "",
              email: emailEl.current?.value ?? "",
              groups: (groupsEl.current?.value ?? "").trim().split(","),
            });

            // appWindow.close();
          }}
        />
      </div>
    </div>
  );
};
export { SettingsComponent as Settings };
export default SettingsComponent;
