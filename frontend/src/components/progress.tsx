import { createSignal, onMount } from "solid-js";
import styles from "./progress.module.scss";


const resolvers: Array<() => void> = [];
function send() {
  if (resolvers.length > 0) {
    resolvers.shift()?.();
  }
}

function receive() {
  return new Promise<void>(resolve => {
    resolvers.push(resolve);
  });
}

const [ width, setWidth ] = createSignal(0);
const [ opacity, setOpacity ] = createSignal(1);

let progress = 0;
let toComplete = 0;

async function loop() {
  const epsilon = 0.0001;
  const speed = 0.05;
  const ospeed = 0.01;
  const dt = 1000 / 60;

  while (true) {
    if (toComplete === 0) {
      await receive();
    }

    while (Math.abs((toComplete !== 0 ? 100 : progress) - width()) > epsilon || (toComplete !== 0 && opacity() > epsilon)) {
      const target = toComplete !== 0 ? 100 : progress;
      setWidth(width() + (target - width()) * (1 - Math.exp(-speed * dt)));

      if (toComplete > 0) {
        setOpacity(opacity() - opacity() * (1 - Math.exp(-ospeed * dt)));
      }

      await new Promise(resolve => setTimeout(resolve, dt));
    }

    if (toComplete === 0) {
      continue;
    }

    toComplete--;
    setWidth(0);
    setOpacity(1);
  }
}

export function increment(value: number) {
  set(progress + value);
}

export function decrement(value: number) {
  set(progress - value);
}

export function complete() {
  toComplete++;
  send();
}

export function set(value: number) {
  progress = Math.min(100, Math.max(0, value));
  send();
}

export function get() {
  return progress;
}


export function Progess() {
  onMount(() => {
    new Promise(_ => loop());
  });

  return (
    <div class={styles.progress}>
      <div class={styles.inner}>
        <div
          class={styles.bar}
          style={{
            width: width() + "%",
            opacity: opacity(),
          }}
        />
      </div>
    </div>
  );
}

export default {
  increment,
  decrement,
  complete,
  set,
  get,
}