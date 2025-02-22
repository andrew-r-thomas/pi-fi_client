import "./App.css";
import { Route, Router } from "@solidjs/router";
import Library from "./routes/Library";
import Album from "./routes/Album";
import { IoPlaySharp, IoPauseSharp, IoPlaySkipForwardSharp } from 'solid-icons/io';

import { createContext, createResource, createSignal, Match, Suspense, Switch } from "solid-js";
// import { invoke } from "@tauri-apps/api/core";

type PlayerData = {
  playing: boolean;
  currentTrack: number;
};

const getPlayerData = async (): Promise<PlayerData> => {
  // return await invoke("setup_player");
  return {
    playing: false,
    currentTrack: 1,
  };
}

function App() {
  const PlayerContext = createContext<PlayerData>();
  const [playerData, { mutate }] = createResource(getPlayerData);
  const [playerBig, setPlayerBig] = createSignal(false);

  return (
    <Suspense>
      <PlayerContext.Provider value={playerData()}>
        <main class="bg-black text-white w-screen h-screen overflow-hidden flex flex-col space-between">
          <div class="p-8 overflow-hidden w-full h-full">
            <Router>
              <Route path="/" component={Library} />
              <Route path="/album/:id" component={Album} />
            </Router>
          </div>
          <div class={`border-y border-white flex flex-row w-full items-center justify-center ${playerBig() ? "h-full" : "h-24"}`}>
            <div class="flex flex-row space-x-4">
              <button onClick={() => mutate((prev) => {
                // PERF: i dont like this
                return { playing: !prev!.playing, currentTrack: prev!.currentTrack }
              }
              )}>
                <Switch>
                  <Match when={playerData()?.playing}>
                    <IoPauseSharp size={32} />
                  </Match>
                  <Match when={!playerData()?.playing}>
                    <IoPlaySharp size={32} />
                  </Match>
                </Switch>
              </button>
              <button>
                <IoPlaySkipForwardSharp size={32} />
              </button>
            </div>
          </div>
        </main>
      </PlayerContext.Provider>
    </Suspense >
  );
}

export default App;
