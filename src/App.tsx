import Patchers from "./components/Patchers";
import Updater from "./components/Updater";

function App() {
  return (
    <div className="container">
      <img src="https://moonlight-mod.github.io/img/logo.png" alt="Moonlight Logo" height="64" width="64" />
      <h1>Welcome to Moonlight.</h1>

      <Updater />
      <Patchers />
    </div>
  );
}

export default App;
