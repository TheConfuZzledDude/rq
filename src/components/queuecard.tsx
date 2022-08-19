import { Queue } from "@/data/queue";
import { User } from "@/data/user";
import { getGravatarUrl, getToastImage, userInQueue } from "@/utilities";
import {
    ClickEvent,
    ControlledMenu,
    MenuItem,
    useMenuState,
} from "@szhsin/react-menu";
import { invoke } from "@tauri-apps/api/tauri";
import { memo } from "preact/compat";

import { useMemo, useRef, useState } from "preact/hooks";

interface QueueCardProps {
    queue: Queue;
    user: User;
    onHide: (id: number) => void;
}

export const QueueCard = memo(({ queue, user, onHide }: QueueCardProps) => {
    const name = queue.name;
    const members  = useMemo(() => queue.members.map((user) =>
        getUserImage(user, "queue-member")
    ), [queue.members]);

    const [showMessages, setShowMessages] = useState(false);
    const inQueue = userInQueue(user, queue);

    const imageUrl = getToastImage(queue);

    const contextMenuRef = useRef(null);
    const messageInputRef = useRef<HTMLTextAreaElement>(null);
    const [menuProps, toggleMenu] = useMenuState();

    return (
        <>
            <div
                onContextMenu={(e) => {
                    e.preventDefault();
                    toggleMenu(true);
                }}
                class={`queue-card ${queue.status === "Started" ? "started" : "open"
                    }`}
            >
                <div class="queue-status">
                    {queue.status === "Started"
                        ? "Started"
                        : inQueue
                            ? "Joined"
                            : "Open"}
                </div>
                <div class="queue-name">
                    {name
                        .split(/(luna)/i)
                        .map((s) =>
                            s.toLowerCase() === "luna" ? (
                                <span class="luna">{s}</span>
                            ) : (
                                s
                            )
                        )}
                </div>
                <div class="queue-image-container">
                    {imageUrl ? (
                        <img
                            class="queue-image"
                            alt="Image corresponding to the hashtag shown in the queue name"
                            src={imageUrl}
                        />
                    ) : (
                        <div class="queue-image-placeholder" />
                    )}
                </div>
                <div class="queue-toolbar">
                    {inQueue ? (
                        <>
                            <button
                                class="queue-button leave"
                                onClick={() => leaveQueue(queue.id)}
                            >
                                Leave Queue
                            </button>
                            {showMessages ? (
                                <button
                                    class="queue-button hidemessage"
                                    onClick={() => setShowMessages(false)}
                                >
                                    Hide Messages
                                </button>
                            ) : (
                                <button
                                    class="queue-button showmessage"
                                    onClick={() => setShowMessages(true)}
                                >
                                    Show Messages
                                </button>
                            )}
                        </>
                    ) : (
                        <button
                            class="queue-button join"
                            onClick={() => joinQueue(queue.id)}
                        >
                            Join Queue
                        </button>
                    )}
                    <button
                        class="queue-button hide"
                        onClick={() => onHide(queue.id)}
                    >
                        Hide Queue
                    </button>
                    <button
                        class={ `queue-button menu ${ menuProps.state === "open" ? "active" : ""}` }
                        onClick={() => toggleMenu(true)}
                        ref={contextMenuRef}
                    />
                    <ControlledMenu
                        {...menuProps}
                        anchorRef={contextMenuRef}
                        menuClassName="context-menu"
                        onMouseLeave={() => toggleMenu(false)}
                        onClose={() => toggleMenu(false)}
                        onItemClick={({ value }: ClickEvent) => {
                            switch (value) {
                                case "start":
                                    startQueue(queue.id);
                                    break;
                                case "nag":
                                    nagQueue(queue.id);
                                    break;
                                case "reset":
                                    resetQueue(queue.id);
                                    break;
                                case "delete":
                                    deleteQueue(queue.id);
                                    break;
                            }
                        }}
                    >
                        {queue.status === "Open" ? (
                            <MenuItem value="start">Start Queue</MenuItem>
                        ) : (
                            <MenuItem value="reset">Reset Queue</MenuItem>
                        )}
                        {queue.status === "Open" && (
                            <MenuItem value="nag">Nag Queue</MenuItem>
                        )}
                        <MenuItem value="delete">Delete Queue</MenuItem>
                    </ControlledMenu>
                </div>
                <div class="queue-members">{members}</div>
            </div>
            {showMessages && (
                <div class="messages-container">
                    {queue.messages.map((message) => {
                        const { sender, content } = message;

                        return (
                            <div
                                class="message-box"
                                key={`${sender}${content}`}
                            >
                                {getUserImage(sender, "message-sender-image")}
                                <span class="message-content">{content}</span>
                            </div>
                        );
                    })}
                    <div class="message-input-container">
                        <textarea
                            class="message-input"
                            ref={messageInputRef}
                            onKeyDown={(e) => {
                                if (e.key === "Enter" && !e.shiftKey) {
                                    e.preventDefault();
                                    if (
                                        messageInputRef.current?.value?.trim()
                                    ) {
                                        messageQueue(
                                            queue.id,
                                            messageInputRef.current.value.trim()
                                        );
                                        messageInputRef.current.value = "";
                                    }
                                }
                            }}
                        />
                        <input
                            class="message-submit"
                            type="submit"
                            onClick={() => {
                                if (
                                    messageInputRef.current &&
                                    messageInputRef.current.value &&
                                    messageInputRef.current.value.trim() !== ""
                                ) {
                                    messageQueue(
                                        queue.id,
                                        messageInputRef.current.value.trim()
                                    );
                                    messageInputRef.current.value = "";
                                }
                            }}
                        />
                    </div>
                </div>
            )}
        </>
    );
});

const leaveQueue = (id: number) => {
    invoke("leave_queue", { id });
};

const joinQueue = (id: number) => {
    invoke("join_queue", { id });
};

const messageQueue = (id: number, content: string) => {
    invoke("message_queue", { id, content });
};

const deleteQueue = (id: number) => {
    invoke("delete_queue", { id });
};

const nagQueue = (id: number) => {
    invoke("nag_queue", { id });
};

const startQueue = (id: number) => {
    invoke("start_queue", { id });
};

const resetQueue = (id: number) => {
    invoke("reset_queue", { id });
};

const getUserImage = (user: User, className: string) => {
    const { username, fullName, email } = user;
    const img = getGravatarUrl(user);
    return (
        <img
            class={className}
            key={`${username},${fullName},${email})`}
            title={fullName}
            alt={`Avatar image of ${fullName}`}
            src={img}
        />
    );
};
