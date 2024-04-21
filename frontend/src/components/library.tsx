import { For, createSignal } from "solid-js";
import { audio, upload_audio } from "../api";
import styles from "./library.module.scss";

export default function Library() {
  const [ hover, setHover ] = createSignal(false);
  
  function onDragOver(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    setHover(true);
  }

  function onDragLeave(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    setHover(false);
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    setHover(false);
    
    if (e.dataTransfer?.files.length !== 1) return;
    const file = e.dataTransfer.files[0];
    upload_audio(file);
  }

  return (
    <div class={styles.library}>
      <div
        class={styles.overlay}
        classList={{ [styles.hover]: hover() }}
        onDragOver={onDragOver}
        onDragLeave={onDragLeave}
        onDrop={onDrop}
      />
      <table
        class={styles.table}
        onDragOver={onDragOver}
        onDragLeave={onDragLeave}
        onDrop={onDrop}
      >
        <thead>
          <tr>
            <th>Title</th>
            <th>Downloads</th>
            <th>Duration</th>
            <th>Author</th>
            <td></td>
            <td></td>
            <td></td>
          </tr>
        </thead>
        <tbody>
          <For each={audio()}>
            {() => <Audio />}
          </For>
        </tbody>
      </table>
    </div>
  );
}


type AudioProps = {};

export function Audio(props: AudioProps) {
  return (
    <tr></tr>
  );
}