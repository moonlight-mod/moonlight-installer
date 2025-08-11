const os = require("node:os");
const fs = require("node:fs");
const path = require("node:path");
const process = require("node:process");

const { MOONLIGHT_INJECTOR, PATCHED_ASAR } = JSON.parse(fs.readFileSync(path.join(__dirname, "moonlight.json")));

const injector = MOONLIGHT_INJECTOR;
require(injector).inject(path.resolve(__dirname, `../${PATCHED_ASAR}`));
