/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";

export const SERVER_URL = "http://192.168.50.68:8080";

render(() => <App />, document.getElementById("root") as HTMLElement);
