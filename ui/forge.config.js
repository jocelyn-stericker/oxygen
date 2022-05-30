/* eslint-env node */
// @ts-check

/** @type {import('@electron-forge/maker-zip').default['config']} */
module.exports = {
  packagerConfig: {
    derefSymlinks: true,
    ignore: [
      "src/(?!(preload\\.js|main\\.js))",
      "node_modules/oxygen-core/target",
      "node_modules/oxygen-core/src",
      "node_modules/oxygen-core/build.rs",
      "node_modules/oxygen-core/Cargo.*",
      "node_modules/.package-lock.json",
      "oxygen\\.sqlite",
      ".postcssrc",
      ".parcel-cache",
      "tailwind.config.js",
    ],
  },
  makers: [
    {
      name: "@electron-forge/maker-zip",
      platforms: ["darwin", "win32", "linux"],
    },
  ],
  publishers: [
    {
      name: "@electron-forge/publisher-github",
      config: {
        repository: {
          owner: "emilyskidsister",
          name: "oxygen",
        },
        prerelease: true,
        draft: true,
      },
    },
  ],
};
