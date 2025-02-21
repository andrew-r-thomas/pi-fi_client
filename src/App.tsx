import "./App.css";
import { Route, Router } from "@solidjs/router";
import Library from "./routes/Library";
import Album from "./routes/Album";

function App() {

  return (
    <main class="bg-black text-white w-screen h-screen overflow-hidden p-8">
      <Router>
        <Route path="/" component={Library} />
        <Route path="/album/:id" component={Album} />
      </Router>
    </main>
  );
}

export default App;
