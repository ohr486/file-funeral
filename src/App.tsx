import { useState } from "react";
import "./App.css";

function App() {
  const [count, setCount] = useState(0);

  return (
    <div className="container">
      <h1>file-funeral</h1>
      <p>クラウドファイル同期アプリケーション</p>

      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
      </div>

      <p className="read-the-docs">
        Tauri + React + TypeScript で開発中
      </p>
    </div>
  );
}

export default App;
