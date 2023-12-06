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
  const needsUpdate = latestVersion !== currentVersion;

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

  function preview(ver: string) {
    if (ver.startsWith("v")) {
      return ver;
    } else {
      return ver.slice(0, 7);
    }
  }

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        justifyContent: "space-between",
        alignItems: "center"
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

      {branch != null && (
        <span>
          Installed version:{" "}
          {currentVersion == null ? "None" : preview(currentVersion)}
        </span>
      )}
      {branch != null && latestVersion != null && (
        <span>Latest version: {preview(latestVersion)}</span>
      )}

      {branch != null && needsUpdate && (
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
          Update moonlight
        </button>
      )}
    </div>
  );
}
