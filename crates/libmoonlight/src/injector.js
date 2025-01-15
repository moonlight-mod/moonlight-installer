const os = require("os");
const path = require("path");
const process = require("process");

function getInjector() {
  if (MOONLIGHT_INJECTOR !== null) return MOONLIGHT_INJECTOR;

  // resolve default path
  switch (os.platform()) {
    case "windows":
      return path.join(process.env.APPDATA, "moonlight-mod", DOWNLOAD_DIR, "injector.js");
    case "darwin":
      return path.join(os.homedir(), "Library", "Application Support", "moonlight-mod", DOWNLOAD_DIR, "injector.js");
    case "linux":
    default:
      return path.join(
        process.env.XDG_CONFIG_HOME ?? path.join(os.homedir(), ".config"),
        "moonlight-mod",
        DOWNLOAD_DIR,
        "injector.js"
      );
  }
}

console.error(`injecting ${getInjector()}`);
require(getInjector()).inject(require("path").resolve(__dirname, `../${PATCHED_ASAR}`));
