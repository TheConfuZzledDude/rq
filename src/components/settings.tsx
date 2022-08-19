import { invoke } from "@tauri-apps/api";
import { appWindow, PhysicalSize } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "preact/hooks";

import "@/css/app.css";
import "@/css/themes/98/98.scss";
import useSize from "@react-hook/size";

interface Settings {
    fullName?: string;
    username?: string;
    email?: string;
    groups?: [string];
}

export const Settings = () => {
    const fullNameEl = useRef<HTMLInputElement>(null);
    const usernameEl = useRef<HTMLInputElement>(null);
    const emailEl = useRef<HTMLInputElement>(null);
    const groupsEl = useRef<HTMLInputElement>(null);

    const mainContentRef = useRef<HTMLDivElement>(null);
    const [width, height] = useSize(mainContentRef);

    const [settings, setSettings] = useState<Settings>({});

    useEffect(() => {
        (async () => {
            appWindow.setSize(
                new PhysicalSize(
                    (await appWindow.innerSize()).width,
                    height * (await appWindow.scaleFactor())
                )
            );
        })();
    }, [height, width]);

    useEffect(() => {
        invoke<Record<string, string>>("fetch_settings").then(setSettings);
    }, []);

    console.log(settings);

    return (
        <div ref={mainContentRef} id="app">
            <div data-tauri-drag-region id="titlebar" class="titlebar">
                <div class="titlebar-logo-container">
                    <img class="titlebar-logo" />
                </div>
                <div
                    class="titlebar-button"
                    id="titlebar-minimize"
                    onClick={appWindow.minimize}
                    alt="minimize"
                />
                <div
                    class="titlebar-button"
                    id="titlebar-close"
                    alt="close"
                    onClick={appWindow.close}
                />
            </div>
            <div class="settings-container">
                <label for="full_name">Full Name</label>
                <input
                    ref={fullNameEl}
                    id="full_name"
                    type="text"
                    class="settings-full-name"
                    value={settings["fullName"]}
                />
                <label for="username">Username</label>
                <input
                    ref={usernameEl}
                    id="username"
                    type="text"
                    class="settings-username"
                    value={settings["username"]}
                />
                <label for="email">Email</label>
                <input
                    ref={emailEl}
                    id="email"
                    type="text"
                    class="settings-email"
                    value={settings["email"]}
                />
                <label for="groups">Groups (Comma Separated)</label>
                <input
                    ref={groupsEl}
                    id="groups"
                    type="text"
                    class="settings-groups"
                    value={settings["groups"]?.join(",")}
                />
                <input
                    type="submit"
                    onClick={() => {
                        invoke("write_settings", {
                            settings: {
                                fullName: fullNameEl.current?.value ?? "",
                                username: usernameEl.current?.value ?? "",
                                email: emailEl.current?.value ?? "",
                                groups: (groupsEl.current?.value ?? "")
                                    .trim()
                                    .split(","),
                            },
                        });

                        console.log({
                            fullName: fullNameEl.current?.value ?? "",
                            username: usernameEl.current?.value ?? "",
                            email: emailEl.current?.value ?? "",
                            groups: (groupsEl.current?.value ?? "")
                                .trim()
                                .split(","),
                        });

                        // appWindow.close();
                    }}
                >
                    Submit
                </input>
            </div>
        </div>
    );
};
