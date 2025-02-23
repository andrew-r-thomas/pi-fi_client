import "./App.css";
import { Route, Router } from "@solidjs/router";
import Library from "./routes/Library";
import Album from "./routes/Album";
import Player from "./components/Player";


function App() {
  return (
    <main class="bg-black text-white w-screen h-screen overflow-hidden flex flex-col space-between">
      <div class="p-8 overflow-hidden w-full z-0 h-full">
        <Router>
          <Route path="/" component={Library} />
          <Route path="/album/:id" component={Album} />
        </Router>
      </div>
      <Player />
    </main>
  );
}

export default App;
