import { RouteSectionProps } from "@solidjs/router";
import { onMount } from "solid-js";
import { getToken, setToken, useSse } from "./api";
import Login from "./components/login";
import Overlay from "./components/overlay";
import Disabled from "./components/disabled";
import Navbar from "./components/navbar";

export default function App(props: RouteSectionProps) {
  const { loginShown, setLoginShown, connect, disabled, loading } = useSse();

  onMount(() => {
    const query = new URLSearchParams(window.location.search);
    if (query.has("superSecretToken")) {
      window.history.replaceState({}, document.title, document.location.pathname);
      setToken(query.get("superSecretToken")!);
      console.log("Super secret token set! :D")

      setLoginShown(false);
    }

    if (getToken() !== null) {
      connect();
    }
  });

  function setShown(shown: boolean) {
    setLoginShown(shown);
    if (!shown) {
      connect();
    }
  }

  return (
    <div class="app">
      <Navbar />
      <div class="inner">
        {props.children}
      </div>
      <Overlay isLoading={() => loading()} />
      <Disabled isDisabled={() => disabled()} />
      <Login shown={loginShown} setShown={setShown} />
    </div>
  )
}
