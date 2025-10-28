import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import MainPage from "./pages/MainPage";
import ConfigPage from "./pages/ConfigPage";

export default function App() {
  const [clientReady, setClientReady] = useState(false);
  const [currentPage, setCurrentPage] = useState<"main" | "config">("main");

  useEffect(() => {
    const init = async () => {
      try {
        await invoke("init_client");
        console.log("Client initialized");
        setClientReady(true);
      } catch (err) {
        console.error("Failed to init client:", err);
      }
    };
    init();
  }, []);

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      {!clientReady && <p>Initializing client...</p>}

      {clientReady && (
        <>
          <div className="buttons" style={{ marginBottom: "1rem" }}>
            <button type="button" onClick={() => setCurrentPage("main")}>Main</button>
            <button type="button" onClick={() => setCurrentPage("config")}>Config</button>
          </div>

          <div className="content">
            {currentPage === "main" && <MainPage />}
            {currentPage === "config" && <ConfigPage />}
          </div>
        </>
      )}
    </main>
  );
}
