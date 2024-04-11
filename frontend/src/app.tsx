import { RouteSectionProps } from "@solidjs/router";
import { createSignal, onMount } from "solid-js";
import Login from "./components/login";
import Overlay from "./components/overlay";

export default function App(props: RouteSectionProps) {
  const [ loginShown, setLoginShown ] = createSignal(window.localStorage.getItem("authorization") === null);

  onMount(() => {
    const query = new URLSearchParams(window.location.search);
    if (!query.has("superSecretToken")) return;

    window.localStorage.setItem("authorization", query.get("superSecretToken")!);
    setLoginShown(false);
  });

  return (
    <div class="app">
      {props.children}
      <Overlay />
      <Login shown={loginShown} setShown={setLoginShown} />
    </div>
  )
}
