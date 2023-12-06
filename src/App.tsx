import "./App.css";
import Patchers from "./Patchers";
import Updater from "./Updater";

function App() {
  return (
    <div className="container">
      <Updater />
      <Patchers />
    </div>
  );
}

export default App;
