import * as studio from "@storyteller/studio";
import { StudioMode, ViewportSize } from "@storyteller/studio";
import { inject } from "@storyteller/framework";
import { LitElement, PropertyValues, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import { CANVAS_PROVIDER, type CanvasProvider } from "./canvas-provider.element";

import styles from "./viewer.element.scss?inline";

@customElement("sts-viewer")
export class ViewerElement extends LitElement {
	static override styles = unsafeCSS(styles);

	// TODO: `<sts-viewer />` and `<sts-studio />` should be separate element
	//       types with different inputs. `<sts-viewer />` should take a BVH
	//       animation and load a simple mannequin asset playing the given
	//       animation in a loop, while `<sts-studio />` should take a scene
	//       asset handle and load it with full editing capabilities.
	//
	//       Both elements should share most of the core functionality here,
	//       which could easily be extracted to a base class for inheritance by
	//       making `#startViewer` an abstract method.
	@property() objectId?: string;
	@property() skyboxId?: string;
	@property({ type: Number }) mode?: StudioMode;

	@inject(CANVAS_PROVIDER)
	_canvasProvider?: CanvasProvider;

	#canvas?: HTMLCanvasElement;
	#resizeObserver?: ResizeObserver;

	override connectedCallback(): void {
		super.connectedCallback();

		this.#resizeObserver = new ResizeObserver(this.#onResize.bind(this));
		this.#resizeObserver.observe(this);

		if (!this._canvasProvider)
			throw new Error(`No provider found for token: ${CANVAS_PROVIDER}`);

		if ((this.#canvas = this._canvasProvider.takeCanvas()))
			this.appendChild(this.#canvas);
	}

	override disconnectedCallback(): void {
		if (this.#canvas)
			this._canvasProvider?.returnCanvas(this.removeChild(this.#canvas));

		this.#resizeObserver?.disconnect();

		super.disconnectedCallback();
	}

	protected override updated(changes: PropertyValues<this>): void {
		if (!studio.isInitialized()) {
			this.#startViewer();
		} else {
			if (changes.has("objectId") && !!this.objectId)
				studio.loadObject(this.objectId);

			if (changes.has("skyboxId") && !!this.skyboxId)
				studio.setSkybox(this.skyboxId);
		}
	}

	protected override render = () => html`
		<slot name="canvas"></slot>
	`;

	#onResize(): void {
		const { width, height } = this.getBoundingClientRect();
		studio.resize(new ViewportSize(width, height));
	}

	#startViewer(): void {
		if (!this.#canvas) return;
		if (!this.objectId) return;
		if (!this.skyboxId) return;
		if (this.mode == null) return;

		const { width, height } = this.getBoundingClientRect();
		try {
			studio.start(
				this.objectId,
				this.skyboxId,
				`#${this._canvasProvider!.canvasId}`,
				"Storyteller Studio",
				new ViewportSize(width, height),
				this.mode,
			);
		} catch (err) {
			if (!(err as Error).message?.startsWith("Using exceptions for control flow"))
				throw err;
		}
	}
}

declare global {
	export interface HTMLElementTagNameMap {
		"sts-viewer": ViewerElement;
	}
}
