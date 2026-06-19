import { render, createSignal } from "solite-runtime";
import "./styles.css";

function App() {
  const [count, setCount] = createSignal(0);

  return (
    <main class="app">
      <h1>Hello from solite</h1>
      <button onClick={() => setCount(count() + 1)}>
        Clicked {count()} times
      </button>
    </main>
  );
}

render(() => App(), __SOL_ROOT__);
