import { inject } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import { ViewportSize } from "@storyteller/studio";
import { LitElement, PropertyValues, html, unsafeCSS } from "lit";

import { CANVAS_PROVIDER, type CanvasProvider } from "./canvas-provider.element";

import styles from "./studio-base.element.scss?inline";

export abstract class StudioBaseElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@inject(CANVAS_PROVIDER)
	_canvasProvider?: CanvasProvider;

	#canvas?: HTMLCanvasElement;
	#resizeObserver?: ResizeObserver;

	abstract onStart(viewportSize: ViewportSize): void;
	abstract onChanges?(changes: PropertyValues<this>): void;

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
			this.#start();
		} else {
			this.onChanges?.(changes);
		}
	}

	#onResize(): void {
		const { width, height } = this.getBoundingClientRect();
		studio.resize(new ViewportSize(width, height));
	}

	#start(): void {
		if (!this.#canvas) return;

		const { width, height } = this.getBoundingClientRect();
		try {
			this.onStart(new ViewportSize(width, height));
		} catch (err) {
			if (!(err as Error).message?.startsWith("Using exceptions for control flow"))
				throw err;
		}
	}

	protected override render = () => html`
		<slot name="canvas"></slot>
	`;
}
