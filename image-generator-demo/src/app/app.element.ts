import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	SceneState,
	type SceneStateEvent,
	type ScreenshotReadyEvent,
	StudioMode,
} from "@storyteller/studio";
import { StudioURLParams, ViewerElement } from "@storyteller/studio-web";
import { match } from "@storyteller/utility";
import { LitElement, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";
import { DebounceSettings, debounce } from "lodash";

import "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/scene-hierarchy";
import "@storyteller/studio-web/transform-toolbar";
import "@storyteller/studio-web/viewer";

import "./app.element.scss"

export enum LCMLiveStatus {
    CONNECTED = "connected",
    DISCONNECTED = "disconnected",
    WAIT = "wait",
    SEND_FRAME = "send_frame",
    TIMEOUT = "timeout",
}
class Debounce {
	static readonly Wait = 1000 / 24;
	static readonly Realtime = {
		leading: false,
		trailing: true,
		maxWait: 1000 / 5,
	};

	static readonly Delayed = {
		leading: false,
		trailing: true,
	};

	static func(config: DebounceSettings) {
		return debounce(() => studio.requestScreenshot(), this.Wait, config);
	}
}

interface IImageInfo {
    prompt?: string;
    negative_prompt?: string;
    seed?: number;
    guidance_scale?: number;
}

// imageinfo class

class ImageInfo {
	prompt?: string;
	negativePrompt?: string;
	seed?: number;
	guidanceScale?: number;

	constructor(data: IImageInfo) {
		this.prompt = data.prompt;
		this.negativePrompt = data.negative_prompt;
		this.seed = data.seed;
		this.guidanceScale = data.guidance_scale;
	}
}

@customElement("app-root")
export class AppElement extends LitElement {
	@state() loading = true;
	@state() imageCaptureUrl?: string;
	@state() realtimeCapture = true;
	@state() lcmLiveStatus = LCMLiveStatus.DISCONNECTED;
	@state() websocket: WebSocket | null = null;
	@state() websocketUUID: string | null = null;
	@state() latestBlob: Blob | null = null;

	params = new StudioURLParams();
	viewer = createRef<ViewerElement>();

	protected override createRenderRoot = () => this;

	override disconnectedCallback(): void {
		if (this.imageCaptureUrl != null)
			URL.revokeObjectURL(this.imageCaptureUrl);

		super.disconnectedCallback();
	}

	@on("scene-state")
	onSceneStateChange({ detail: state }: SceneStateEvent): void {
		console.log("scene state change:", SceneState[state]);

		this.loading = match (state, {
			[SceneState.Active]: () => false,
			_: () => true,
		});
	}

	@on("entity-changes")
	onStudioChanges(): void {
		this.requestScreenshot();
	}

	requestScreenshot = Debounce.func(Debounce.Realtime);

	@on("screenshot-ready")
	onScreenshotReady({ detail: screenshot }: ScreenshotReadyEvent) {
		const prevUrl = this.imageCaptureUrl;
		// const blob = new Blob([screenshot.data.buffer], { type: "image/png" });
		const blob = new Blob([screenshot.data.buffer], {type : 'image/jpeg'});
		this.latestBlob = blob;
		// const blob = new Blob (["iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII="]);

		// const innerSend = (blob: Blob | { [key: string]: any }) : void => {
		// 	if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
		// 		if (blob instanceof Blob) {
		// 			this.websocket.send(blob);
		// 		} else {
		// 			this.websocket.send(JSON.stringify(blob));
		// 		}
		// 	} else {
		// 		console.log("WebSocket not connected durign innerSend with state" + this.lcmLiveStatus + " and readyState " + this.websocket?.readyState + " and blob " + blob + " and websocket " + this.websocket + " and websocket.readyState " + this.websocket?.readyState);
		// 	}
		// };

		// if websocket is disconnected, connect
		if (this.lcmLiveStatus === LCMLiveStatus.DISCONNECTED) {
			const userId = crypto.randomUUID();
			this.websocketUUID = userId;
			this.websocket = new WebSocket("wss://b3000.pancake-chimaera.ts.net/api/ws/" + userId);
			this.websocket.onopen = () => {
				console.log("Marking connection to websocket as open for user " + userId);
				this.lcmLiveStatus = LCMLiveStatus.CONNECTED;
			};
			this.websocket.onclose = () => {
				console.log("Marking connection to websocket as closed for user " + userId);
				this.lcmLiveStatus = LCMLiveStatus.DISCONNECTED;
			};
			this.websocket.onmessage = (event) => {
				const data = JSON.parse(event.data);
				switch (data.status) {
					case "connected":
						this.lcmLiveStatus = LCMLiveStatus.CONNECTED;
						break;
					case "send_frame":
						this.lcmLiveStatus = LCMLiveStatus.SEND_FRAME;
						const default_prompt = "Portrait of The Joker halloween costume, face painting, with , glare pose, detailed, intricate, full of colour, cinematic lighting, trending on artstation, 8k, hyperrealistic, focused, extreme details, unreal engine 5 cinematic, masterpiece"
						const default_negative_prompt = "black and white, blurry, low resolution, pixelated,  pixel art, low quality, low fidelity"
						const imageInfo = new ImageInfo({
							prompt: default_prompt,
							negative_prompt: default_negative_prompt,
							seed: 12345,
							guidance_scale: 1.2,
						});
						const streamData = [this.latestBlob];
						console.log("Sending next frame with streamData" + streamData + " and websocket " + this.websocket + " and websocket.readyState " + this.websocket?.readyState)
						this.websocket?.send(JSON.stringify({ status: "next_frame" }));
						this.websocket?.send(JSON.stringify({ prompt: default_prompt }));

						for (const d of streamData) {
							if (d instanceof Blob) {
								this.websocket?.send(d);
							} else {
								this.websocket?.send(JSON.stringify(d));
							}
						}
						break;
					case "wait":
						this.lcmLiveStatus = LCMLiveStatus.WAIT;
						break;
					case "timeout":
						console.log("timeout");
						this.lcmLiveStatus = LCMLiveStatus.TIMEOUT;
						break;
					case "error":
						console.log(data.message);
						this.lcmLiveStatus = LCMLiveStatus.DISCONNECTED;
						// lcmLiveStatus.set(LCMLiveStatus.DISCONNECTED);
						// streamId.set(null);
						// reject(new Error(data.message));
						break;
				}
			};
		}


		if (this.lcmLiveStatus === LCMLiveStatus.CONNECTED) {

		} else {
			console.log("WebSocket not connected with state" + this.lcmLiveStatus + " and readyState " + this.websocket?.readyState);
		}

		this.imageCaptureUrl = URL.createObjectURL(blob);
		console.log(`New blob URL: ${this.imageCaptureUrl}`);

		if (prevUrl != null)
			URL.revokeObjectURL(prevUrl);
	}

	toggleRealtimeCapture(event: InputEvent): void {
		const { checked } = event.target as HTMLInputElement;
		this.realtimeCapture = checked;

		const config = checked ? Debounce.Realtime : Debounce.Delayed;
		this.requestScreenshot = Debounce.func(config);
	}

	protected override render = () => html`
		<sts-canvas-provider class="app-root">
			<sts-viewer
				${ref(this.viewer)}
				class="studio-scene"
				objectId="base-human-female.gltf"
				skyboxId="test_scene"
				.mode=${StudioMode.Editor}
			></sts-viewer>

			${this.imageCaptureUrl != null ? html`
				<img
					class="image-capture"
					src=${this.imageCaptureUrl}
					alt="Image capture"
				/>
			` : nothing}

			<sts-scene-hierarchy class="hierarchy h-start v-start"></sts-scene-hierarchy>
			<sts-transform-toolbar class="h-center v-start"></sts-transform-toolbar>
			<div class="config v-start h-end">
				<label class="checkbox">
					<input
						type="checkbox"
						?checked=${this.realtimeCapture}
						@input=${this.toggleRealtimeCapture}
					/>
					Realtime Capture
				</label>
			</div>

			${this.loading ? html`
				<!-- TODO: Animated spinner or something -- talk to our designers -->
				<div class="loading">
					<h1>Loading</h1>
				</div>
			` : nothing}
		</sts-canvas-provider>
	`
}
