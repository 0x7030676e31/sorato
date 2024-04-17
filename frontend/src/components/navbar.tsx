import { useLocation, useNavigate } from "@solidjs/router";
import { Show } from "solid-js";
import { isHead, clients, audio, groups, actors } from "../api";
import styles from "./navbar.module.scss";

export default function Navbar() {
  const location = useLocation();
  const navigate = useNavigate();

  return (
    <div class={styles.navbar}>
      <div class={styles.tab} classList={{ [styles.active]: location.pathname === "/sorato" }} onClick={() => navigate("/sorato")}>
        Clients
        <div class={styles.badge}>{clients().length}</div>
      </div>
      <div class={styles.tab} classList={{ [styles.active]: location.pathname === "/sorato/library" }} onClick={() => navigate("/sorato/library")}>
        Library
        <div class={styles.badge}>{audio().length}</div>
      </div>
      <div class={styles.tab} classList={{ [styles.active]: location.pathname === "/sorato/groups" }} onClick={() => navigate("/sorato/groups")}>
        Groups
        <div class={styles.badge}>{groups().length}</div>
      </div>
      <Show when={isHead()}>
        <div class={styles.tab} classList={{ [styles.active]: location.pathname === "/sorato/actors" }} onClick={() => navigate("/sorato/actors")}>
          Actors
          <div class={styles.badge}>{actors().length}</div>
        </div>
        <div class={styles.tab} classList={{ [styles.active]: location.pathname === "/sorato/server" }} onClick={() => navigate("/sorato/server")}>
          Server
        </div>
      </Show>
    </div>
  );
}