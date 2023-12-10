import { MoonlightError, ErrorCode, Branch } from "../types";
import React from "react";
import { invoke } from "@tauri-apps/api";
import useEvent from "../util";

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
  const [patching, setPatching] = React.useState<Branch | null>(null);

  useEvent<MoonlightError>("error", (err) => {
    setError(err);
  });

  useEvent<Branch>("patching_install", (branch) => {
    setPatching(branch);
  });

  useEvent<Branch>("unpatching_install", (branch) => {
    setPatching(branch);
  });

  if (error == null) return null;

  switch(error.code) {
    case ErrorCode.WindowsFileLock:
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
              if (patching != null) {
                await invoke("kill_discord", {
                  branch: patching
                });
              }
            }}
          >
            Force close Discord
          </button>
        </ErrorBoxInner>
      );
    case ErrorCode.MacOSNoPermission:
      return (
        <ErrorBoxInner header="Please modify your system settings." close={() => setError(null)}>
          <p>
            moonlight is unable to modify your Discord installation. This is because your MacOS system privacy settings doesn't allow us to do so.
          </p>

          <p>
            You can fix this via a pop-up you should've gotten, or by going to 
            System Settings &gt; Privacy & Security &gt; App Management and allowing moonlight installer.
          </p>

        </ErrorBoxInner>
      )
    default:
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
