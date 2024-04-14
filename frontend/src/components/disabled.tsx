import { Accessor } from "solid-js";
import styles from "./disabled.module.scss";

export default function Disabled({ isDisabled }: { isDisabled: Accessor<boolean> }) {
  return (
    <div class={styles.overlay} classList={{ [styles.hidden]: !isDisabled() }}>
      <div class={styles.inner}>
        <h1 class={styles.title}>Your account has been temporarily disabled.</h1>
        <p class={styles.subtitle}>The app will automatically reload when your account is re-enabled.</p>
      </div>
    </div>
  );
}