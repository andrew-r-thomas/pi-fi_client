import "./App.css";
import { Route, Router } from "@solidjs/router";
import Library from "./routes/Library";
import Album from "./routes/Album";

function App() {

  return (
    <main class="bg-black text-white w-screen h-screen overflow-hidden flex flex-col space-between">
      <div class="p-8 overflow-hidden w-full h-full">
        <Router>
          <Route path="/" component={Library} />
          <Route path="/album/:id" component={Album} />
        </Router>
      </div>
      <div class="border-2 border-indigo-500 h-24">
        this is where the player will go
      </div>
    </main>
  );
}

export default App;
