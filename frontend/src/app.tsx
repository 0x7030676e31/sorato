import { RouteSectionProps } from "@solidjs/router";
import { createSignal } from "solid-js";
import Login from "./components/login";
import Overlay from "./components/overlay";

export default function App(props: RouteSectionProps) {
  const [ loginShown, setLoginShown ] = createSignal(window.localStorage.getItem("authorization") === null);

  return (
    <div class="app">
      {props.children}
      <Overlay />
      <Login shown={loginShown} setShown={setLoginShown} />
    </div>
  )
}
