
/// <reference types='vitest' />
import { defineConfig } from "vite";

import { nxViteTsPaths } from "@nx/vite/plugins/nx-tsconfig-paths.plugin";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

import mergeAssetsDirs from "./vite-plugins/multiple-assets";

export default defineConfig({
	cacheDir: "../node_modules/.vite/hello-web",
	server:{
		port: 4200,
		host: "localhost",
	},
	preview:{
		port: 4300,
		host: "localhost",
	},
	plugins: [
		nxViteTsPaths(),
		wasm(),
		topLevelAwait(),
		mergeAssetsDirs([
			"hello-web/public",
			"dist/hello-world/public",
		]),
	],
});
