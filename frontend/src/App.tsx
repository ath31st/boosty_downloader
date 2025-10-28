import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
	const [clientReady, setClientReady] = useState(false);

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
			{clientReady && <p>Client is ready!</p>}
		</main>
	);
}

export default App;
