import { useCallback, useEffect, useRef, useState } from "preact/hooks";
import { listen } from "@tauri-apps/api/event";
import { QueueCard } from "@/components/queuecard";
import { invoke } from "@tauri-apps/api/tauri";
import { Queue } from "@/data/queue";
import { User } from "@/data/user";
import useSize from "@react-hook/size";
import { appWindow, PhysicalSize, WebviewWindow } from "@tauri-apps/api/window";

import rqLogo from "icons/128x128.png";

import "@/css/app.css";
import "@szhsin/react-menu/dist/core.css";

// import "@/css/themes/default_theme.scss";
import "@/css/themes/98/98.scss";

import {
    ClickEvent,
    ControlledMenu,
    MenuItem,
    useMenuState,
} from "@szhsin/react-menu";
import { exit } from "@tauri-apps/api/process";
import { fs } from "@tauri-apps/api";
import { BaseDirectory } from "@tauri-apps/api/fs";

type Theme = "Win98" | "ClassicQ3" | "Modern";
interface FetchQueuesResponse {
    queues: [Queue];
    currentUser: User;
    theme: Theme;
}

export const App = () => {
    console.log("rerendering app");
    const [queues, setQueues] = useState<Record<number, Queue>>();

    const [currentUser, setCurrentUser] = useState<User>({
        username: "",
        email: "",
        fullName: "",
    });
    const [theme, setTheme] = useState<Theme>("Modern");
    const [hiddenQueues, setHiddenQueues] = useState(new Set());

    const hideQueue = useCallback(
        (id: number) => {
            const newSet = new Set(hiddenQueues);
            newSet.add(id);
            setHiddenQueues(newSet);
        },
        [hiddenQueues]
    );

    const mainContentRef = useRef<HTMLDivElement>(null);
    const [width, height] = useSize(mainContentRef);

    const [menuProps, toggleMenu] = useMenuState();
    const contextMenuRef = useRef(null);

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
        let unlistenQueuesUpdated: (() => void) | null = null;
        // let unlistenMousePosition: (() => void) | null = null;
        async function fetchData() {
            unlistenQueuesUpdated = await listen<FetchQueuesResponse>(
                "queues_updated",
                ({ payload: { queues, currentUser } }) => {
                    setQueues(queues);
                    setCurrentUser(currentUser);
                }
            );
            await invoke("get_queues");
        }
        fetchData();
        return () => {
            unlistenQueuesUpdated && unlistenQueuesUpdated();
            // unlistenMousePosition && unlistenMousePosition();
        };
    }, []);

    useEffect(() => {
        const f = async () => {
            fs.readTextFile("config.json", { dir: BaseDirectory.Config })
        }
    }, []);

    return (
        <div ref={mainContentRef} id="app">
            <div data-tauri-drag-region id="titlebar" class="titlebar">
                <div class="titlebar-logo-container">
                    <img class="titlebar-logo" src={rqLogo} />
                </div>
                <div
                    class={`titlebar-button ${
                        menuProps.state === "open" ? "active" : ""
                    }`}
                    id="titlebar-menu"
                    ref={contextMenuRef}
                    onClick={() => toggleMenu(true)}
                    alt="menu"
                />
                <div
                    class="titlebar-button"
                    id="titlebar-minimize"
                    onClick={appWindow.minimize}
                    alt="minimize"
                />
                <div
                    class="titlebar-button"
                    id="titlebar-maximize"
                    onClick={appWindow.maximize}
                    alt="maximize"
                />
                <div
                    class="titlebar-button"
                    id="titlebar-close"
                    alt="close"
                    onClick={appWindow.hide}
                />
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
                            new WebviewWindow("new_queue", {
                                url: "new_queue.html",
                                decorations: false,
                            });
                            break;
                        }
                        case "restore": {
                            setHiddenQueues(new Set());
                            break;
                        }
                        case "settings": {
                            new WebviewWindow("settings", {
                                url: "settings.html",
                                decorations: false,
                            });
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
            <div class="queues-container">
                {Object.entries(queues ?? {})
                    ?.filter(([, queue]) => !hiddenQueues.has(queue.id))
                    .map(([, queue]) => (
                        <QueueCard
                            key={queue.id}
                            queue={queue}
                            user={currentUser}
                            onHide={hideQueue}
                        />
                    ))}
            </div>
        </div>
    );
};
