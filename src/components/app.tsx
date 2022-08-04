import { useCallback, useEffect, useRef, useState } from "preact/hooks";
import { listen } from "@tauri-apps/api/event";
import { QueueCard } from "@/components/queuecard";
import { invoke } from "@tauri-apps/api/tauri";
import { Queue } from "@/data/queue";
import { User } from "@/data/user";
import useSize from "@react-hook/size";
import {
    appWindow,
    LogicalSize,
    PhysicalSize,
    WebviewWindow,
} from "@tauri-apps/api/window";

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
import { tauri } from "@tauri-apps/api";

export const App = () => {
    const [queues, setQueues] = useState<[Queue]>();

    const currentUser: User = {
        username: "ZacFre",
        fullName: "Zachary Freed",
        email: "Zachary.Freed@softwire.com",
    };

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

    const [menuProps, toggleMenu] = useMenuState({ transition: true });
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
            unlistenQueuesUpdated = await listen<[Queue]>(
                "queues_updated",
                ({ payload }) => {
                    setQueues(payload);
                }
            );
            // unlistenMousePosition = await listen<{x: number, y:number}>(
            //     "mouse_position",
            //     ({payload: {x, y}}) => {
            //         const elem = document.elementFromPoint(x,y);
            //         if (elem === null || elem?.id === "app") {
            //             console.log(WebviewWindow.getByLabel('main'));
            //         }
            //     }
            // )
            await invoke("get_queues");
        }
        fetchData();
        return () => {
            unlistenQueuesUpdated && unlistenQueuesUpdated();
            // unlistenMousePosition && unlistenMousePosition();
        };
    }, []);

    return (
        <div ref={mainContentRef} id="app">
            <div
                data-tauri-drag-region
                id="titlebar"
                class="titlebar"
            >
                <div class="titlebar-logo-container">
                    <img class="titlebar-logo" src={rqLogo} />
                </div>
                <div
                    class="titlebar-button"
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
                    onClick={appWindow.close}
                />
            </div>
            <ControlledMenu
                {...menuProps}
                anchorRef={contextMenuRef}
                menuClassName="window"
                onMouseLeave={() => toggleMenu(false)}
                onClose={() => toggleMenu(false)}
                onItemClick={({ value }: ClickEvent) => {
                    switch (value) {
                        case "new":
                            {
                                new WebviewWindow(
                                    "newQueue",
                                    {
                                        url: "new_queue.html",
                                        decorations: false,
                                    }
                                );
                            }
                            break;
                    }
                }}
            >
                <MenuItem value="new">New Queue</MenuItem>
            </ControlledMenu>
            <div class="queues-container">
                {Object.entries(queues ?? [])
                    .filter(([, queue]) => !hiddenQueues.has(queue.id))
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

