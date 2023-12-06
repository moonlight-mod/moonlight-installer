import React from "react";
import { MoonlightBranch } from "../types";
import { invoke } from "@tauri-apps/api";

export default function Updater() {
  const [branch, setBranch] = React.useState<MoonlightBranch | null>(null);
  const [currentVersion, setCurrentVersion] = React.useState<string | null>(
    null
  );
  const [latestVersion, setLatestVersion] = React.useState<string | null>(null);
  const [locked, setLocked] = React.useState<boolean>(false);

  React.useEffect(() => {
    async function updateBranch() {
      const branch: MoonlightBranch = await invoke("get_moonlight_branch");
      setBranch(branch);
    }

    updateBranch();
  }, []);

  async function updateVersions() {
    const currentVersion: string | null = await invoke(
      "get_downloaded_moonlight"
    );
    setCurrentVersion(currentVersion);

    const latestVersion: string | null = await invoke(
      "get_latest_moonlight_version",
      { branch }
    );
    setLatestVersion(latestVersion);
  }

  React.useEffect(() => {
    if (branch != null) updateVersions();
  }, [branch]);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        justifyContent: "space-between",
        alignItems: "flex-start"
      }}
    >
      {branch != null && (
        <select
          value={branch}
          onChange={async (e) => {
            const branch = e.target.value as MoonlightBranch;
            await invoke("set_moonlight_branch", { branch });
            setBranch(branch);
          }}
        >
          <option value={MoonlightBranch.Stable}>Stable</option>
          <option value={MoonlightBranch.Nightly}>Nightly</option>
        </select>
      )}

      <span>Current: {currentVersion}</span>
      <span>Latest: {latestVersion}</span>

      {branch != null && (
        <button
          disabled={locked}
          onClick={async () => {
            setLocked(true);
            try {
              await invoke("download_moonlight", { branch });
            } finally {
              setLocked(false);
              await updateVersions();
            }
          }}
        >
          Update
        </button>
      )}
    </div>
  );
}
