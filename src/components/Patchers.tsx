import { invoke } from "@tauri-apps/api";
import { DetectedInstall } from "../types";
import React from "react";
import { emit, listen } from "@tauri-apps/api/event";

function Install({ install }: { install: DetectedInstall }) {
  const [patched, setPatched] = React.useState<boolean | null>(null);
  const [locked, setLocked] = React.useState<boolean>(false);
  const [installedVersion, setInstalledVersion] = React.useState<string | null>(
    null
  );

  React.useEffect(() => {
    async function updatePatched() {
      const patched: boolean = await invoke("is_install_patched", { install });
      setPatched(patched);
    }

    updatePatched();
  }, []);

  // how the fuck do Tauri events work
  React.useEffect(() => {
    invoke("get_downloaded_moonlight").then((result) => {
      setInstalledVersion(result as string | null);
    });

    let unlisten = listen("installed_version_changed", (event) => {
      setInstalledVersion(event.payload as string | null);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  async function togglePatch() {
    setLocked(true);

    try {
      if (patched) {
        await invoke("unpatch_install", { install });
      } else {
        await invoke("patch_install", { install });
      }

      setPatched(!patched);
    } catch (e) {
      await emit("error", e);
    } finally {
      setLocked(false);
    }
  }

  return (
    <div className="install">
      <h3>Discord {install.branch}</h3>

      {install != null && (
        <button
          onClick={togglePatch}
          disabled={locked || installedVersion == null}
        >
          {patched ? "Unpatch" : "Patch"}
        </button>
      )}
    </div>
  );
}

export default function Patchers() {
  const [installs, setInstalls] = React.useState<DetectedInstall[]>([]);
  async function updateInstalls() {
    const installs: DetectedInstall[] = await invoke("detect_installs");
    setInstalls(installs);
  }

  React.useEffect(() => {
    updateInstalls();
  }, []);

  return (
    <div className="install-list">
      {installs.map((install, i) => (
        <Install install={install} key={i} />
      ))}
    </div>
  );
}
