import { useEffect, useState } from "preact/hooks";
import preactLogo from "./assets/preact.svg";
import "./app.css";
import { appWindow } from "@tauri-apps/api/window";
import { emit, listen } from '@tauri-apps/api/event'

interface Queue {
    id: number,
    name: string,
    status: number,
    members: [User],
    messages: [Message],
}

interface Message {
    content: string,
    sender: User
}

interface User {
    username: string,
    fullName: string,
    email: string,
}
//  User {
//     pub(crate) username: String,
//     pub(crate) full_name: String,
//     pub(crate) email: String,
// }

// #[derive(Serialize, Deserialize, FromPrimitive, Debug)]
// pub(crate) enum QueueStatus {
//     Open = 0,
//     Started = 1,
//     Closed = 2,
// }

// #[derive(Serialize, Deserialize,Debug)]
// pub(crate) struct Queue {
//     pub(crate) id: u64,
//     pub(crate) name: String,
//     pub(crate) status: QueueStatus,
//     pub(crate) members: Vec<User>,
//     pub(crate) messages: Vec<Message>,
// }

// #[derive(Serialize, Deserialize,Debug)]
// pub(crate) struct Message {
//     pub(crate) content: String,
//     pub(crate) sender: User,
// }

export function App() {
    const [queues, setQueues] = useState<[Queue]>()

    console.log("Test");
    useEffect(() => {
        (async () => {
            const unlisten = await listen<[Queue]>("queues_updated", ({payload}) => setQueues(payload))
        })();
    }, [])

    return (
        <>
            <div data-tauri-drag-region class="titlebar">
                <div
                    class="titlebar-button"
                    id="titlebar-minimize"
                    onClick={appWindow.minimize}
                >
                    <img
                        src="https://api.iconify.design/mdi:window-minimize.svg"
                        alt="minimize"
                    />
                </div>
                <div
                    class="titlebar-button"
                    id="titlebar-maximize"
                    onClick={appWindow.maximize}
                >
                    <img
                        src="https://api.iconify.design/mdi:window-maximize.svg"
                        alt="maximize"
                    />
                </div>
                <div
                    class="titlebar-button"
                    id="titlebar-close"
                    onClick={appWindow.close}
                >
                    <img
                        src="https://api.iconify.design/mdi:close.svg"
                        alt="close"
                    />
                </div>
            </div>
            <div> 
                {JSON.stringify(queues)}
            </div>
        </>
    );
}
