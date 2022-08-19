import "preact/debug";
import { render } from "preact";
import { App } from "@/components/app";
import "@/css/index.css";
import { appWindow } from "@tauri-apps/api/window";

render(<App />, document.getElementsByTagName("body")[0] as HTMLElement);
