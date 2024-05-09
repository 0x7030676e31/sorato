import { For, createSignal, Show } from 'solid-js';
import { audio, clients, upload_audio, uploading, actorsMinimal, actorId, isHead, delete_audio as remove_audio } from "../api";
import { AiTwotoneDelete } from "solid-icons/ai";
import { timeify } from "../utils";
import styles from "./library.module.scss";

export default function Library() {
  const [ hover, setHover ] = createSignal(false);
  let input: HTMLInputElement | undefined;
  
  function open() {
    input?.click();
  }

  function onUpload(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files?.length !== 1 || uploading()) return;
    
    const file = target.files[0];
    upload_audio(file);
  }

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
    <div
      class={styles.library}
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
    >
      <input type="file" accept="audio/*" class={styles.input} ref={input} onChange={onUpload} />
      <div class={styles.overlay} classList={{ [styles.hover]: hover() && !uploading() }} />
      <Show when={audio().length !== 0} fallback={<Fallback open={open} />}>
        <div
          class={styles.content}
          classList={{ [styles.uploading]: uploading() }}
        >
          <div class={styles.header}>Title</div>
          <div class={styles.header}>Downloads</div>
          <div class={styles.header}>Length</div>
          <div class={styles.header}>Author</div>
          <div class={styles.header}></div>
          <div class={styles.header}></div>
          <div class={styles.header}></div>
          <For each={audio()}>
            {entry => <Audio {...entry} />}
          </For>
        </div>
      </Show>
    </div>
  );
}

function Fallback({ open }: { open: () => void }) {
  return (
    <div class={styles.fallback}>
      <h1>(╯°□°)╯︵ ┻━┻</h1>
      <h2> No media available. </h2>
      <h3 onClick={open}> Upload some media to get started. </h3>
    </div>
  );
}

function Audio(props: Audio) {
  return (
    <div class={styles.entry}>
      <div>{props.title}</div>
      <div>{props.downloads.length} / {clients().length}</div>
      <div>{timeify(props.length)}</div>
      <div>{actorsMinimal().find(a => a.id === props.author)?.name ?? "Unknown"}</div>
      <div></div>
      <div></div>
      <div class={styles.iconWrapper}>
        <div  
          onClick={() => (props.author === actorId() || isHead()) && remove_audio(props.id)}
          class={`${styles.icon} ${styles.delete}`}
          classList={{
            [styles.disabled]: props.author !== actorId() && !isHead(),
          }}
        >
          <AiTwotoneDelete />
        </div>
      </div>
    </div>
  );
}