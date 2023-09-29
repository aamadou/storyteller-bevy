// NOTE: This is an inline fork of this npm package:
// https://github.com/nguyenbatranvan/vite-multiple-assets
//
// The published version at time of writing doesn't include a mime type for
// WGSL, which is fixed in this copy. This module can be removed and replaced
// with the npm dependency if/when this PR is accepted:
// https://github.com/nguyenbatranvan/vite-multiple-assets/pull/5

import {PluginOption, ViteDevServer} from "vite";
import {ServerMiddleWare} from "./server";
import {buildMiddleWare} from "./build";

export default function DynamicPublicDirectory(assets: string[]): PluginOption {
    return {
        configureServer(server: ViteDevServer) {
            return ServerMiddleWare(server, assets)
        },
        async writeBundle(options) {
            buildMiddleWare(options, assets)
        },
        name: "dynamic assets"
    };
}
