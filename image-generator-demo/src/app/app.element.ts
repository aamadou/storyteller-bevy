import { debounce, drag, on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	SceneState,
	type SceneStateEvent,
	type ScreenshotReadyEvent,
	StudioMode,
} from "@storyteller/studio";
import { match } from "@storyteller/utility";
import { LitElement, type PropertyValues, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { styleMap } from "lit/directives/style-map.js";
import { classMap } from "lit/directives/class-map.js";

import "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/scene-hierarchy";
import "@storyteller/studio-web/studio";
import "@storyteller/studio-web/transform-toolbar";

import "./app.element.scss"

@customElement("app-root")
export class AppElement extends LitElement {
	@state() imageCaptureUrl?: string;

	@state() hierarchyWidth = 256;
	@state() resizing = false;

	@state() loading = true;

	@state() prompt = "";
	@state() seed = randomSeed();

	protected override createRenderRoot = () => this;

	override disconnectedCallback(): void {
		if (this.imageCaptureUrl != null)
			URL.revokeObjectURL(this.imageCaptureUrl);

		super.disconnectedCallback();
	}

	protected override willUpdate(changes: PropertyValues<this>): void {
		if (changes.has("prompt") || changes.has("seed") || changes.has("imageCaptureUrl")) {
			// TODO: Send parameters to model
			console.log("changes:", {
				prompt: this.prompt,
				seed: this.seed,
				imageCaptureUrl: this.imageCaptureUrl,
			});
		}

		super.willUpdate(changes);
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
	@debounce(1000 / 24)
	onStudioChanges(): void {
		studio.requestScreenshot();
	}

	@on("viewport-resized")
	// This handler requires a longer debounce to avoid crashing the render piptline
	@debounce(1000 / 5)
	onViewportResized(): void {
		studio.requestScreenshot();
	}

	@on("screenshot-ready")
	onScreenshotReady({ detail: screenshot }: ScreenshotReadyEvent) {
		const prevUrl = this.imageCaptureUrl;

		const blob = new Blob([screenshot.data.buffer], { type: "image/png" });
		this.imageCaptureUrl = URL.createObjectURL(blob);

		if (prevUrl != null)
			URL.revokeObjectURL(prevUrl);
	}

	updatePrompt(event: Event): void {
		this.prompt = (event.target as HTMLInputElement).value;
	}

	refreshSeed(): void {
		this.seed = randomSeed();
	}

	stopTextInputBubbling(event: KeyboardEvent): void {
		// We want modifier keys like "Alt" to always be received by the Bevy
		// canvas because they affect the camera controller behavior, but we do
		// NOT want Bevy to receive typing inputs like W, A, S, and D while the
		// user is trying to type into the Prompt field.
		if (/^(Alt|Control|Shift)$/.test(event.key))
			return;

		event.stopPropagation();
	}

	onResizeStart(): void {
		this.resizing = true;
	}

	onResizeEnd(): void {
		this.resizing = false;
	}

	onResize(event: PointerEvent): void {
		let updated = this.hierarchyWidth + event.movementX;
		this.hierarchyWidth = Math.max(Math.min(updated, 512), 128);
	}

	protected override render = () => html`
		<sts-canvas-provider class="app-root">
			<div
				class="hierarchy-wrapper"
				style=${styleMap({ width: `${this.hierarchyWidth}px` })}
			>
				<sts-scene-hierarchy class="hierarchy"></sts-scene-hierarchy>
				<div
					class=${classMap({
						"hierarchy-resize-handle": true,
						"resizing": this.resizing,
					})}
					${drag({
						start: this.onResizeStart,
						move: this.onResize,
						end: this.onResizeEnd,
					})}
				></div>
			</div>

			<div class="split-screen">
				<sts-studio
					class=${classMap({
						"studio-scene": true,
						"resizing": this.resizing,
					})}
					objectId="base-human-female.gltf"
					skyboxId="test_scene"
					.mode=${StudioMode.Editor}
				>
					<sts-transform-toolbar
						class="xform-toolbar h-center v-start"
					></sts-transform-toolbar>
				</sts-studio>

				${this.imageCaptureUrl != null ? html`
					<img
						class="image-capture"
						src=${this.imageCaptureUrl}
						alt="Image capture"
					/>
				` : html`
					<div class="image-capture placeholder"></div>
				`}

				<div class="config config-prompt h-center v-end"
					@keydown=${this.stopTextInputBubbling}
					@keyup=${this.stopTextInputBubbling}
				>
					<input
						class="config-field prompt"
						type="text"
						aria-label="Prompt"
						placeholder="Enter a prompt to customize the output"
						.value=${this.prompt}
						@change=${this.updatePrompt}
						@keydown=${(event: KeyboardEvent) => {
							if (event.key === "Enter") {
								event.preventDefault();
								this.updatePrompt(event);
							}
						}}
					/>
					<div class="config-field-group">
						<input
							class="config-field seed"
							type="number"
							aria-label="Seed"
							.value=${this.seed.toString()}
							min="1"
							max="99999"
						/>
						<button
							class="seed-refresh"
							@click=${this.refreshSeed}
						>
							<sts-icon icon="rotate"></sts-icon>
						</button>
					</div>
				</div>
			</div>

			${/*
			FIXME: Real-time capture is not really feasible until we can come up
			       with a more elegant way to exclude the world grid, gizmos, etc.
			       from the captured image

			<div class="config v-start h-end">
				<label class="config-checkbox">
					<input
						type="checkbox"
						?checked=${this.realtimeCapture}
						@input=${this.toggleRealtimeCapture}
					/>
					Realtime Capture
				</label>
			</div>
			*/
			nothing}

			${this.loading ? html`
				<!-- TODO: Animated spinner or something -- talk to our designers -->
				<div class="loading">
					<h1>Loading</h1>
				</div>
			` : nothing}
		</sts-canvas-provider>
	`
}

function randomSeed(): number {
	return Math.round(Math.random() * 99999);
}
