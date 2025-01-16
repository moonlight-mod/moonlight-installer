const os = require("node:os");
const fs = require("node:fs");
const path = require("node:path");
const process = require("node:process");

function getInjector(override) {
  if (override !== null && fs.existsSync(override)) return override;

  // resolve default path
  switch (os.platform()) {
    case "win32":
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

const injector = getInjector(MOONLIGHT_INJECTOR);
require(injector).inject(path.resolve(__dirname, `../${PATCHED_ASAR}`));
