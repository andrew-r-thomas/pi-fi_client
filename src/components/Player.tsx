import { Channel, invoke } from "@tauri-apps/api/core";
import { IoPauseSharp, IoPlaySharp, IoPlaySkipForwardSharp } from "solid-icons/io";
import { createSignal, Match, onMount, Show, Switch } from "solid-js";
import { createStore } from "solid-js/store";
import { SERVER_URL } from "..";

type PlayerData = {
  playing: boolean;
  current_track: {
    track_title: string;
    artist_title: string;
    cover_art_id: number;
  } | null;
};

type PlayerUpdateMsg = {
  event: "UpdatePlaying";
  data: {
    playing: boolean;
  };
} | {
  event: "UpdateCurrentTrack";
  data: {
    current_track: {
      track_title: string;
      artist_title: string;
      cover_art_id: number;
    };
  };
};

function Player() {
  const [playerBig, setPlayerBig] = createSignal(false);
  const [playerData, setPlayerData] = createStore<PlayerData>({ playing: false, current_track: null });

  onMount(() => {
    const channel = new Channel<PlayerUpdateMsg>();
    channel.onmessage = (message) => {
      switch (message.event) {
        case "UpdatePlaying":
          setPlayerData("playing", message.data.playing);
          break;
        case "UpdateCurrentTrack":
          console.log(JSON.stringify(message.data));
          setPlayerData("current_track", message.data.current_track);
          break;
      }
    };
    invoke("setup_player", { channel });
  });

  return (
    <div
      onClick={() => setPlayerBig(!playerBig())}
      class={`border-y fixed bottom-0 bg-black p-4 transition-all ease-in-out duration-100 border-white z-40 flex flex-row w-full max-w-full items-center justify-center ${playerBig() ? "h-full" : "h-24"}`}>
      <Show when={playerData.current_track !== null} fallback={<div>uhhh</div>}>
        <div class="flex flex-row justify-between w-full">
          <div class="flex flex-row space-x-4 max-w-2/3 overflow-hidden">
            <img class="w-14 h-14" src={`${SERVER_URL}/get-image?id=${playerData.current_track?.cover_art_id}`} />
            <div class="flex flex-col w-full overflow-hidden">
              <p class="font-bold font-serif text-xl text-nowrap overflow-hidden text-ellipsis w-full">{playerData.current_track?.track_title}</p>
              <p>{playerData.current_track?.artist_title}</p>
            </div>
          </div>
          <div class="relative flex flex-row space-x-4">
            <button onClick={(e) => {
              e.stopPropagation();
              invoke("toggle_playing");
            }
            }>
              <Switch>
                <Match when={playerData.playing}>
                  <IoPauseSharp size={32} />
                </Match>
                <Match when={!playerData.playing}>
                  <IoPlaySharp size={32} />
                </Match>
              </Switch>
            </button>
            <button onClick={(e) => e.stopPropagation()}>
              <IoPlaySkipForwardSharp size={32} />
            </button>
          </div>
        </div>
      </Show>
    </div>
  )
}

export default Player;
