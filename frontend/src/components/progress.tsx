import { createSignal } from "solid-js";
import styles from "./progress.module.scss";

const [ progress, setProgress ] = createSignal(0);

export function setValue(value: number) {
  setProgress(Math.min(100, Math.max(0, value)));
}

export function addValue(value: number) {
  setValue(progress() + value);
}

export function reset() {
  setValue(0);
}

export function Progess() {
  return (
    <div class={styles.progress}>
      <div class={styles.bar} style={{ width: `${progress()}%` }} classList={{ [styles.done]: progress() === 100 }} />
    </div>
  );
}