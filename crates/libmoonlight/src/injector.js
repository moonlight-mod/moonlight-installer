const os = require("node:os");
const fs = require("node:fs");
const path = require("node:path");
const process = require("node:process");

const moonlightDir = () => {
  switch (os.platform()) {
    case "win32":
      return path.join(process.env.APPDATA, "moonlight-mod");
    case "darwin":
      return path.join(os.homedir(), "Library", "Application Support", "moonlight-mod");
    case "linux":
    default:
      return path.join(process.env.XDG_CONFIG_HOME ?? path.join(os.homedir(), ".config"), "moonlight-mod");
  }
};

const parse = ({ pathStr, relativeTo }) => {
  switch (relativeTo) {
    case "MOONLIGHT":
      return path.join(moonlightDir(), pathStr);
    default:
      return pathStr;
  }
};

const { MOONLIGHT_INJECTOR, PATCHED_ASAR } = JSON.parse(fs.readFileSync(path.join(__dirname, "moonlight.json")));

const injector = parse(MOONLIGHT_INJECTOR);
require(injector).inject(path.resolve(__dirname, `../${PATCHED_ASAR}`));
