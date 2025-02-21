import { createResource, For, Suspense } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { A } from "@solidjs/router";

type LibraryData = {
  albums: AlbumData[];
};
type AlbumData = {
  id: number;
  title: string;
  artist_name: string;
};

const getLibrary = async (): Promise<LibraryData> => await invoke("get_library");

function Library() {
  const [lib] = createResource(getLibrary);

  return (
    <div class="flex flex-col w-full h-full space-y-8">
      <h1 class="text-4xl font-bold font-serif">Library</h1>
      <div class="flex flex-col space-y-4">
        <div>
          <h2 class="text-2xl font-bold">Albums</h2>
          <hr />
        </div>
        <div class="flex flex-row space-x-4 overflow-x-scroll w-full">
          <Suspense>
            <For each={lib()?.albums}>
              {(album) => (
                <A href={`/album/${album.id}`} class="max-w-1/3 border-blue-500">
                  <h3 class="text-md font-bold font-serif text-nowrap text-ellipsis overflow-hidden">{album.title}</h3>
                  <p>by {album.artist_name}</p>
                </A>
              )}
            </For>
          </Suspense>
        </div>
      </div>
    </div>
  )
}

export default Library;
