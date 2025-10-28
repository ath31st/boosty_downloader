import { useState } from "react";
import styles from "./MainPage.module.css";

export default function MainPage() {
  const [url, setUrl] = useState("");
  const [logs, setLogs] = useState<string[]>([]);
  const [progress, setProgress] = useState(0);
  const [downloading, setDownloading] = useState(false);

  const startDownload = async () => {
    if (!url) return;
    setLogs([]);
    setProgress(0);
    setDownloading(true);

    const total = 100;
    for (let i = 1; i <= total; i++) {
      await new Promise((res) => setTimeout(res, 50));
      setProgress(i);
      setLogs((prev) => [...prev, `Скачано ${i}%`]);
    }

    setLogs((prev) => [...prev, "Скачивание завершено!"]);
    setDownloading(false);
  };

  return (
    <div className="main-page">
      <h2>Скачать контент</h2>

      <div className={styles.inputGroup}>
        <input
          type="text"
          placeholder="Введите URL"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          disabled={downloading}
        />
        <button onClick={startDownload} disabled={downloading || !url}>
          Скачать
        </button>
      </div>

      <div className={styles.logs}>
        {logs.map((line, idx) => (
          <p key={idx}>{line}</p>
        ))}
      </div>

      <div className={styles.progressBar}>
        <div
          className={styles.progressFill}
          style={{ width: `${progress}%` }}
        ></div>
      </div>
    </div>
  );
}
