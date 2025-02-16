<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { upload } from "@tauri-apps/plugin-upload";

  const addTracks = async () => {
    const selected = await open({
      multiple: false,
      directory: true,
    });
    if (!selected) return;

    /*

    NOTE:
    - ok so just for you to pick up where you left off, right now we want to focus
      on full library ingest, as we want our metadata system to be good enough
      that most users will do a big ingest up front, and then add albums
      individually as they buy
    - we want to nail down how much to do on the client vs the server

     */

    let proms = [];
    let headers = new Map<string, string>();
    headers.set("Content-Type", "audio/flac");
    for (let path of selected) {
      let p = upload(
        "http://localhost:8080/upload",
        path,
        ({ progress, total }) => {
          console.log(`uploaded ${progress}/${total} of ${path}`);
        },
        headers,
      );

      proms.push(p);
    }

    let resps = await Promise.all(proms);
    console.log(`resps: ${resps}`);
  };
</script>

<main class="bg-slate-900 h-screen w-screen text-white flex flex-col p-4">
  <h1>pi-fi</h1>
  <button onclick={addTracks} class="border-red-600 border-2 h-fit w-fit">
    ingest
  </button>
</main>
