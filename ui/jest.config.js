/* eslint-env node */

module.exports = {
  transform: {
    "^.+\\.(t|j)sx?$": ["@swc-node/jest"],
  },
  modulePathIgnorePatterns: ["out"],
  runner: "@kayahr/jest-electron-runner",
  testEnvironment: "@kayahr/jest-electron-runner/environment",
};
