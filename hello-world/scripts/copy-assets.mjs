import { promises as fs } from "node:fs";
import path from "node:path";
import url from "node:url";

async function main() {
	const __dirname = url.fileURLToPath(new URL(".", import.meta.url));
	const assetsSrc = path.resolve(__dirname, "../assets");
	const assetsDest = path.resolve(__dirname, "../../dist/hello-world/public/assets");

	await fs.cp(assetsSrc, assetsDest, { recursive: true });
}

main();
