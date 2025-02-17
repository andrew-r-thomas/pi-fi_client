<script lang="ts">
  /*

    TODO: experiment with doing things vanilla js style and moving as much as
    possible to the rust side of things

  */

  import { open } from "@tauri-apps/plugin-dialog";
  import { upload } from "@tauri-apps/plugin-upload";
  import { readDir, stat } from "@tauri-apps/plugin-fs";
  import { extname, join } from "@tauri-apps/api/path";
  import { fetch } from "@tauri-apps/plugin-http";

  import { Plus, LoaderCircle } from "lucide-svelte";

  let uploading = $state(false);
  let uploadProgress = $state(0);
  let uploadTotal = $state(0);

  type Album = {
    id: number;
    title: string;
  };

  let waiting = $state(false);
  let album = $state<Album | null>(null);

  const addAlbum = async () => {
    const selected = await open({
      multiple: false,
      directory: true,
    });
    if (!selected) return;

    const dir = await readDir(selected);

    let tracks = [];
    for (const entry of dir) {
      const path = await join(selected, entry.name);
      const ext = await extname(path);
      if (ext != "flac") continue;

      const md = await stat(path);
      uploadTotal += md.size;
      tracks.push(path);
    }

    waiting = true;
    const albumId = await upload(
      "http://localhost:8080/upload-track",
      tracks[0],
    );
    console.log(`album id: ${albumId}`);
    const albumResp = await fetch(`http://localhost:8080/album?id=${albumId}`);
    album = await albumResp.json();
    waiting = false;

    uploading = true;
    await new Promise((r) => setTimeout(r, 1000));
    uploading = false;

    uploadTotal = 0;
    uploadProgress = 0;
  };
</script>

<main class="bg-slate-900 h-screen w-screen text-white flex flex-col p-4">
  <h1 class="text-4xl font-bold">Library</h1>
  <button
    onclick={addAlbum}
    disabled={waiting}
    class="border-indigo-700 border-2 h-fit w-fit flex flex-row space-x-2 px-4 py-2 rounded-md"
  >
    {#if waiting}
      <LoaderCircle class="animate-spin" />
    {:else}
      <Plus />
    {/if}
    <span>add album</span>
  </button>
  {#if album !== null}
    <div class="flex flex-col">
      <p>{album.title}</p>
      {#if uploading}
        <p class="flex flex-col">
          <span class="font-bold">progress</span>
          <span>{(uploadProgress / uploadTotal) * 100}%</span>
        </p>
      {/if}
    </div>
  {/if}
</main>
