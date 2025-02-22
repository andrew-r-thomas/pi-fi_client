import "./App.css";
import { createAsyncStore, Route, Router } from "@solidjs/router";
import Library from "./routes/Library";
import Album from "./routes/Album";
import { IoPlaySharp, IoPauseSharp } from 'solid-icons/io';

import { createContext, createSignal, Match, Switch } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

type PlayerData = {
  playing: boolean;
  currentTrack: number;
};

const getPlayerData = async (): Promise<PlayerData> => {
  return await invoke("setup_player");
}

function App() {
  const [playing, setPlaying] = createSignal(false);
  const PlayerContext = createContext();
  const [playerStore, setPlayerStore] = createAsyncStore(getPlayerData);

  return (
    <main class="bg-black text-white w-screen h-screen overflow-hidden flex flex-col space-between">
      <div class="p-8 overflow-hidden w-full h-full">
        <Router>
          <Route path="/" component={Library} />
          <Route path="/album/:id" component={Album} />
        </Router>
      </div>
      <div class="border-2 border-indigo-500 h-24 flex flex-row w-full items-center justify-center">
        <button onClick={() => setPlaying(!playing())}>
          <Switch>
            <Match when={playing()}>
              <IoPauseSharp />
            </Match>
            <Match when={!playing()}>
              <IoPlaySharp size={32} />
            </Match>
          </Switch>
        </button>
      </div>
    </main>
  );
}

export default App;
