import * as studio from "@storyteller/studio";
import { LitElement, type PropertyValues, html, nothing, unsafeCSS, TemplateResult } from "lit";
import { type Ref, createRef, ref } from "lit/directives/ref.js";
import { customElement, property, state } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { styleMap } from "lit/directives/style-map.js";
import { type ScreenshotReadyEvent } from "@storyteller/studio";
import { DialogElement } from "@storyteller/studio-ui/dialog";
import { bind, on } from "@storyteller/framework";
import { v4 as uuidv4 } from "uuid";
import {
	EngineCompositer,
	EngineCompositerResponse
} from "./api/EngineCompositer"
import { VideoStylePreview } from "./api/VideoStylePreview";

import styles from "./style-workspace.element.scss?inline";

import "./bubble-select.element";
import "@storyteller/studio-ui/button";
import { STYLE_OPTIONS, type VstStyle } from "./inspector-tabs/style-tab";


@customElement("sts-style-workspace")
export class StyleWorkspace extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ attribute: false })
	trigger?: Ref<HTMLElement>;

	@state() sourceBlob?: Blob;
	@state() resultBlob?: Blob;

	@state() sourceImageUrl?: string;
	@state() resultImageUrl?: string;

	@state() selectedStyle: VstStyle = "anime_2d_flat";
	@state() prompt = "";
	@state() negPrompt = "";

	@property() sceneToken = "";

	#temporaryVideoPreviewRef = createRef<DialogElement>();

	protected override willUpdate(changes: PropertyValues<this>): void {
		if (changes.has("trigger")) {
			const prevTrigger = changes.get("trigger");
			prevTrigger?.value?.removeEventListener("click", this.onDialogOpen);

			this.trigger?.value?.addEventListener("click", this.onDialogOpen);
		}

		super.willUpdate(changes)
	}

	override disconnectedCallback(): void {
		this.trigger?.value?.removeEventListener("click", this.onDialogOpen);

		if (this.sourceImageUrl) {
			URL.revokeObjectURL(this.sourceImageUrl);
			this.sourceImageUrl = undefined;
		}

		if (this.resultImageUrl) {
			URL.revokeObjectURL(this.resultImageUrl);
			this.resultImageUrl = undefined;
		}

		super.disconnectedCallback();
	}

	requestScreen = () => {
		studio.requestScreenshot();
		console.log("🟢");
	}

	onDialogOpen = () => {
		studio.requestScreenshot();
	}

	@on("window:screenshot-ready")
	async onScreenshotReady (event: ScreenshotReadyEvent) {
		console.log("🟣",event.detail.data);
		const file = new File(
			[event.detail.data],
			"studio-scene-screen.png",
			{ type: "image/png" },
		);
		this.sourceBlob = file;

		const prevSourceUrl = this.sourceImageUrl;
		this.sourceImageUrl = URL.createObjectURL(this.sourceBlob);

		if (prevSourceUrl)
			URL.revokeObjectURL(prevSourceUrl);

		this.resultImageUrl = undefined;

		try {
			this.resultBlob = await VideoStylePreview({
				video: file,
				request:
					JSON.stringify({
						"style": this.selectedStyle,
						"positive_prompt": this.prompt,
						"negative_prompt": this.negPrompt,
					})
			});
			console.log("🟡", this.resultBlob);
		}
		catch (err) {
			// TODO: Show error message
			console.error(err);
			return;
		}

		const prevResultUrl = this.resultImageUrl;
		this.resultImageUrl = URL.createObjectURL(this.resultBlob);

		if (prevResultUrl)
			URL.revokeObjectURL(prevResultUrl);
	}

	stopTextInputBubbling(event: KeyboardEvent): void {
		// We want modifier keys like "Alt" to always be received by the Bevy
		// canvas because they affect the camera controller behavior, but we do
		// NOT want Bevy to receive typing inputs like W, A, S, and D while the
		// user is trying to type into the Prompt field.
		if (/^(Alt|Control)$/.test(event.key))
			return;

		event.stopPropagation();
	}

	@on("window:scene-uploaded") getSceneToken({ detail }) {
		console.log("👺", detail);
		this.sceneToken = detail;
	}

	compositorEnqueue() {
		console.log("🧑🏻‍🎤 scene token", this.sceneToken);

		if (this.sceneToken) {
			EngineCompositer("",{
				uuid_idempotency_token: uuidv4(),
				media_file_token: this.sceneToken,
			}).then((res: EngineCompositerResponse) => {
				console.log("👻",res);
			}); 
		}

	}


	protected override render = () => html`
		<div class="video-style-preview__page_container">
			<div class="video-style-preview__previews">
				<div class="video-style-preview__panel">
					<h2 class="video-style-preview__heading preview-top-heading">
						Raw Preview
					</h2>
					<div
						class="
							video-style-preview__image
							video-style-preview__image--source"
						style=${styleMap({
							"background-image": this.sourceImageUrl
								? `url(${this.sourceImageUrl})`
								: undefined,
						})}
						src=${ifDefined(this.sourceImageUrl)}
					>
						${!this.sourceImageUrl ? html`
							<h2 class="video-style-preview__heading">
								Loading...
							</h2>
						` : nothing}
					</div>
				</div>
				<svg class="video-style-preview__arrow">
					<line x1="2" y1="15" x2="28" y2="15"/>
					<polyline points="18,4 28,15 18,26"/>
				</svg>
				<div class="video-style-preview__panel">
					<h2 class="video-style-preview__heading preview-top-heading">
						Styled Preview
					</h2>
					<div
						class="
							video-style-preview__image
							video-style-preview__image--result"
						style=${styleMap({
							"background-image": this.resultImageUrl
								? `url(${this.resultImageUrl})`
								: undefined,
						})}
					>
						${!this.resultImageUrl ? html`
							<h2 class="video-style-preview__heading">
								Loading...
							</h2>
						` : nothing}
					</div>
				</div>
			</div>
			<div class="video-style-preview__input-set">
				<sts-bubble-select
					.options=${STYLE_OPTIONS}
					.value=${bind(this, "selectedStyle")}
				></sts-bubble-select>
				<div class="video-style-preview__prompts">
					<div class="video-style-preview__form-field">
						<label
							class="video-style-preview__label"
							for="video-style-preview-prompt"
						>
							Enter a Prompt
						</label>
						<textarea
							id="video-style-preview-prompt"
							class="video-style-preview__input"
							name="prompt"
							cols="60"
							rows="8"
							.value=${this.prompt}
							@input=${(event: InputEvent) => {
								this.prompt = (event.target as HTMLTextAreaElement).value;
							}}
						></textarea>
					</div>
					<div class="video-style-preview__form-field">
						<label
							class="video-style-preview__label"
							for="video-style-preview-neg-prompt"
						>
							Negative Prompt
						</label>
						<textarea
							id="video-style-preview-neg-prompt"
							class="video-style-preview__input"
							name="neg-prompt"
							cols="60"
							rows="8"
							placeholder=${"Type here to filter out the things you don't "
								+ "want in your scene..."}
							.value=${this.negPrompt}
							@input=${(event: InputEvent) => {
								this.negPrompt = (event.target as HTMLTextAreaElement).value;
							}}
						></textarea>
					</div>
				</div>
				<div class="video-style-preview__buttons">
					<sts-button @click=${this.requestScreen}>
						Update preview
					</sts-button>
					<sts-button primary @click=${this.compositorEnqueue}>
						Generate movie
					</sts-button>
				</div>
			</div>
			<div class="video-style-preview__go-back">
				<sts-button>Go back</sts-button>
			</div>
		</div>
	`
}
