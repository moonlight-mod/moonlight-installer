import { listen } from "@tauri-apps/api/event";
import React from "react";

export default function useEvent<T>(
  event: string,
  callback: (data: T) => void
) {
  React.useEffect(() => {
    const unlisten = listen(event, (data) => {
      callback(data.payload as T);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, [event, callback]);
}
