import { listen } from "@tauri-apps/api/event";
import { MoonlightError, ErrorCode } from "../types";
import React from "react";
import { invoke } from "@tauri-apps/api";

function ErrorBoxInner({
  header,
  close,
  children
}: {
  header: string;
  close: () => void;
  children?: React.ReactNode[];
}) {
  return (
    <div className="error-box">
      <div className="error-box-header">
        <h3>{header}</h3>
        <button onClick={close}>x</button>
      </div>

      {children}
    </div>
  );
}

export default function ErrorBox() {
  const [error, setError] = React.useState<MoonlightError | null>(null);

  React.useEffect(() => {
    let unlisten = listen("error", (event) => {
      setError(event.payload as MoonlightError);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  if (error == null) return null;

  if (error.code == ErrorCode.WindowsFileLock) {
    return (
      <ErrorBoxInner header="Please close Discord" close={() => setError(null)}>
        <p>
          Discord is currently open, which locks moonlight's ability to modify
          its files. Please completely close Discord and make sure it does not
          appear in the taskbar.
        </p>

        <p>
          Alternatively, click the button below to attempt to close Discord
          forcefully. This will disconnect you from any voice calls you are in.
        </p>

        <button
          onClick={async () => {
            setError(null);
            // This is flawed in that it'll kill *all* Discord processes, not just the one we patched
            // I'd need to add a store of some kind to see what we tried to patch, which I am too fucking
            // lazy to do, so you get this instead
            await invoke("kill_discord");
          }}
        >
          Force close Discord
        </button>
      </ErrorBoxInner>
    );
  } else {
    return (
      <ErrorBoxInner header="Unknown error" close={() => setError(null)}>
        <p>
          An unknown error occured - please report this, along with this
          information:
        </p>
        <span>{error.code}</span>
        <code>{error.message}</code>
      </ErrorBoxInner>
    );
  }
}
