import { render } from "preact";
import { Settings } from "@/components/settings";
import "@/css/index.css";

render(<Settings />, document.getElementsByTagName("body")[0] as HTMLElement);
