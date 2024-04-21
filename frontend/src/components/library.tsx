import { For, createSignal } from "solid-js";
import { audio, clients, upload_audio, uploading, actorsMinimal } from "../api";
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
    
    if (e.dataTransfer?.files.length !== 1 || uploading()) return;
    const file = e.dataTransfer.files[0];
    upload_audio(file);
  }

  return (
    <div class={styles.library}>
      <div
        class={styles.overlay}
        classList={{ [styles.hover]: hover() && !uploading() }}
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
            <th>Author</th>
            <th>Downloads</th>
            <th>Length</th>
            <th></th>
            <th></th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <For each={audio()}>
            {audio => <Audio {...audio} />}
          </For>
        </tbody>
      </table>
    </div>
  );
}


export function Audio(props: Audio) {
  return (
    <tr>
      <td>{props.title}</td>
      <td>{actorsMinimal().find(a => a.id === props.author)?.name ?? "Unknown"}</td>
      <td>{props.downloads.length} / {clients().length}</td>
      <td>{props.length}</td>
      <td></td>
      <td></td>
      <td></td>
    </tr>
  );
}