import { Accessor, createSignal, onMount } from "solid-js";
import { authorize, setToken } from '../api';
import styles from "./login.module.scss";

export default function Login({ shown, setShown }: { shown: Accessor<boolean>, setShown: (value: boolean) => void }) {
  const [ code, setCode ] = createSignal("");
  const [ error, setError ] = createSignal("");
  const [ loading, setLoading ] = createSignal(false);
  
  onMount(() => {
    const query = new URLSearchParams(window.location.search);
    if (!query.has("code")) return;

    setCode(query.get("code")!);
    proceed();

    history.replaceState({}, "", window.location.pathname);
  });

  const is_disabled = () => code().length < 16 || loading() || error().length !== 0;

  function on_key_down(e: KeyboardEvent) {
    if (e.key === "Enter") proceed();
  }

  async function proceed() {
    if (is_disabled()) return;
    
    setLoading(true);
    let response: Response;

    try {
      response = await authorize(code());
    } catch (err) {
      setError((err as any).toString());
      setLoading(false);
      return;
    }

    if (response.ok) {
      setToken(await response.text()); 
      setShown(false);
      return;
    }
    
    const err = await response.text().catch(() => "An error has occurred");
    setError(err);  
    setLoading(false);
  }

  return (
    <div class={styles.login} classList={{ [styles.shown]: shown() }}>
      <h1> Sorato </h1>
      <p> Login using one time verification code </p>
      <div class={styles.code} classList={{ [styles.error]: error().length !== 0, [styles.loading]: loading() }}>
        <input type="text" disabled={loading()} placeholder="Enter your verification code" onKeyDown={on_key_down} value={code()} onInput={(e) => {
          setCode(e.currentTarget.value);
          setError("");
        }} />
      </div>
      <p class={styles.error}> {error()} </p>
      <button classList={{ [styles.disabled]: is_disabled() }} onClick={proceed}> Proceed </button>
    </div>
  );
}
