import { Accessor } from "solid-js";
import styles from "./overlay.module.scss";

export default function Overlay({ isLoading }: { isLoading: Accessor<boolean> }) {
  return (
    <div class={styles.overlay} classList={{ [styles.hidden]: !isLoading() }}>
      <svg class={styles.spinner} viewBox="0 0 50 50">
        <circle class={styles.path} cx="25" cy="25" r="20" fill="none" stroke-width="5" />
      </svg>
    </div>
  );
}