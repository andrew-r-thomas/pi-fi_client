import { useParams } from "@solidjs/router";
import { invoke } from "@tauri-apps/api/core";
import { Index, Suspense, createResource } from "solid-js";

type Album = {
  title: string;
  artist_name: string;
  artist_id: number;
  tracks: Track[];
};
type Track = {
  id: number;
  title: string;
  track_number: number;
};

const getAlbum = async (id: number): Promise<Album> => await invoke("get_album", { id });

function Album() {
  const { id } = useParams();
  const [album] = createResource(Number(id), getAlbum);

  const playTrack = (id: number) => invoke("play_track", { id });

  return (
    <div class="flex flex-col space-y-8 h-full w-full">
      <Suspense>
        <div>
          <h1
            class="text-4xl font-serif font-bold text-nowrap text-ellipsis overflow-hidden"
          >
            {album()?.title}
          </h1>
          <p class="text-xl font-bold">{album()?.artist_name}</p>
          <hr />
        </div>
        <div class="flex flex-col space-y-4 overflow-y-scroll">
          <Index each={album()?.tracks}>
            {(track) => (
              <button onClick={() => playTrack(track().id)} class="flex flex-row space-x-2 items-center">
                <span>{track().track_number}</span>
                <h2 class="font-bold text-xl font-serif text-nowrap text-ellipsis overflow-hidden">{track().title}</h2>
              </button>
            )}
          </Index>
        </div>
      </Suspense>
    </div>
  )
}

export default Album;
