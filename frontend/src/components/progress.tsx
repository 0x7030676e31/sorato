import { createSignal } from "solid-js";
import styles from "./progress.module.scss";

const [ progress, setProgress ] = createSignal(0);
const [ completed, setCompleted ] = createSignal(false);

export function increment(value: number) {
  set(progress() + value);
}

export function decrement(value: number) {
  set(progress() - value);
}

export function set(value: number) {
  if (timeout !== null) {
    setCompleted(false);
    clearTimeout(timeout);
    timeout = null;
  }

  setProgress(Math.min(100, Math.max(0, value)));
}

export function get() {
  return progress();
}

let timeout: number | null = null;
export function complete() {
  timeout = setTimeout(() => {
    setCompleted(false);
    timeout = null;
  }, 500);
  
  setCompleted(true);
  setProgress(0);
}

export function Progess() {
  return (
    <div class={styles.progress}>
      <div
        class={styles.bar}
        style={{ width: `${progress()}%` }}
        classList={{
          [styles.completed]: completed(),
        }}
      />
    </div>
  );
}

export default {
  increment,
  decrement,
  set,
  get,
  complete,
}