export async function checkForUpdate(): Promise<string | undefined> {
  try {
    const res = await fetch(
      'https://api.github.com/repos/ath31st/boosty_downloader/releases/latest',
      {
        headers: { Accept: 'application/vnd.github+json' },
      },
    );

    if (res.ok) {
      const data = await res.json();
      const match =
        typeof data.tag_name === 'string'
          ? data.tag_name.match(/^v(\d+\.\d+\.\d+)$/)
          : null;
      const tag = match?.[1];

      console.log(`Latest version: ${tag}`);

      return tag;
    }
  } catch (e) {
    console.error(e);
  }
}
