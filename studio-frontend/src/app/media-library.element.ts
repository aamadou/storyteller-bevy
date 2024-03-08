import { inject, on } from "@storyteller/framework";
import {
	OVERLAY_PROVIDER,
	type OverlayHandle,
	type OverlayProvider,
} from "@storyteller/studio-ui/overlay-provider";
import {
	type MediaFile,
	MediaFileType,
	StorytellerApi,
} from "@storyteller/studio-web";
import { LitElement, unsafeCSS, type PropertyValues, html } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";
import { until } from "lit/directives/until.js";

import dialogStyles from "./media-library.element.scss?inline";

@customElement("sts-media-library")
export class MediaLibraryElement extends LitElement {
	@property({ attribute: false })
	trigger = createRef<HTMLElement>();

	@inject(OVERLAY_PROVIDER)
	overlay!: OverlayProvider;

	#overlayHandle?: WeakRef<OverlayHandle>;
	#dialogRef = createRef<MediaLibraryDialogElement>();

	open(): void {
		this.#overlayHandle = this.overlay.attach(this, html`
			<sts-media-library-dialog
				${ref(this.#dialogRef)}
				@close=${() => this.close()}
			></sts-media-library-dialog>
		`, {
			containerStyles: {
				display: "flex",
				alignItems: "center",
				justifyContent: "center",
			},
		});
	}

	close(): void {
		if (this.#overlayHandle)
			this.overlay.detach(this.#overlayHandle);
	}

	protected override updated(changes: PropertyValues<this>): void {
		if (changes.has("trigger")) {
			const prevTrigger = changes.get("trigger");
			if (prevTrigger?.value) {
				prevTrigger.value.removeEventListener("click", this.onTriggerClick);
			}

			const trigger = this.trigger.value;
			if (!trigger) return;

			trigger.addEventListener("click", this.onTriggerClick);
		}

		super.updated(changes);
	}

	override disconnectedCallback(): void {
		this.trigger.value?.removeEventListener("click", this.onTriggerClick);
		super.disconnectedCallback();
	}

	onTriggerClick = (): void => {
		this.open();
	}
}

@customElement("sts-media-library-dialog")
export class MediaLibraryDialogElement extends LitElement {
	static override styles = unsafeCSS(dialogStyles);

	@property({ reflect: true })
	override role = "dialog";

	@state()
	mediaFiles?: Promise<MediaFile[]>;

	@inject(StorytellerApi)
	_api!: StorytellerApi;

	override async connectedCallback(): Promise<void> {
		this.mediaFiles = this._api
			.session()
			.then(session => {
				if (!session.logged_in || !session.user?.username)
					return Promise.reject(new Error("Not logged in"));

				return this._api.listMediaFiles({
					username: session.user.username,
					filterMediaType: [
						MediaFileType.Bvh,
						MediaFileType.Glb,
						MediaFileType.Gltf,
						MediaFileType.ScnRon,
					],
				});
			})
			.then(result => Promise.all(
				result.results.map(({ token }) => this._api.mediaFile(token))
			))
			.catch(err => {
				console.error(err);
				return [];
			});

		super.connectedCallback();
	}

	@on("window:click")
	onWindowClick(event: PointerEvent): void {
		if (!event.composedPath().includes(this))
			this.dispatchEvent(new CustomEvent("close"));
	}

	protected override render = () => html`
		${until(
			this.mediaFiles?.then(files => html`
				<pre>${JSON.stringify(files, null, "  ")}</pre>
			`),
			html`<span>Loading...</span>`
		)}
	`;
}
