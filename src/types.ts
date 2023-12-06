export enum MoonlightBranch {
  Stable = "Stable",
  Nightly = "Nightly"
}

export enum InstallType {
  Windows = "Windows",
  MacOS = "MacOS",
  Linux = "Linux"
}

export enum Branch {
  Stable = "Stable",
  PTB = "PTB",
  Canary = "Canary",
  Development = "Development"
}

export type DetectedInstall = {
  install_type: InstallType;
  branch: Branch;
  path: string;
};
