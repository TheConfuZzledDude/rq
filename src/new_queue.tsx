import { render } from "preact";
import { NewQueue } from "@/components/new_queue";
import "@/css/index.css";

render(<NewQueue />, document.getElementsByTagName("body")[0] as HTMLElement);
