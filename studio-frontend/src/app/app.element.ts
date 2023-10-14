import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import { TransformSpace, ViewportSize } from "@storyteller/studio";
import { match } from "@storyteller/utility";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";

import "./button.element";
import "./help.element";
import "./icon.element";

import styles from "./app.element.scss?inline";

// FIXME: Figure out how to generate the correct code from `wasm_bindgen`
enum TransformType {
	TRANSLATE = 0b0001,
	ROTATE    = 0b0010,
	SCALE     = 0b0100,
}

@customElement("app-root")
export class AppElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() showHelp = false;

	@state() transformSpace = TransformSpace.Local;
	@state() transformType = TransformType.ROTATE | TransformType.TRANSLATE;

	#studioCanvas?: HTMLCanvasElement | null;

	override connectedCallback(): void {
		this.#studioCanvas = this.#initStudio();
		super.connectedCallback();
	}

	@on("window:resize")
	onWindowResize(): void {
		const width = window.innerWidth;
		const height = window.innerHeight;
		studio.resize(new ViewportSize(width, height));
	}

	@on("keydown")
	@on("keyup")
	pipeKeyboardEvents(event: KeyboardEvent): void {
		// When the user interacts with one of the web UI elements, the Bevy
		// canvas will lose focus, preventing it from receiving keyboard events.
		// As a workaround, we can listen for any events that have bubbled up to
		// the app root and repeat them for the canvas.
		//
		// This does unfortunately burden the web front-end with some additional
		// responsibility: when we handle a keyboard event and _don't_ want the
		// Bevy app to receive it, we'll need to call `event.stopPropagation()` in
		// the handler to prevent it from reaching this listener.
		const {
			type,
			code, charCode, key, keyCode, location, repeat,
			altKey, ctrlKey, metaKey, shiftKey,
		} = event;

		this.#studioCanvas?.dispatchEvent(new KeyboardEvent(type, {
			code, charCode, key, keyCode, location, repeat,
			altKey, ctrlKey, metaKey, shiftKey,
			bubbles: true,
			cancelable: true,
			composed: true,
		}));
	}

	toggleHelp(): void {
		this.showHelp = !this.showHelp;
	}

	onSpaceChange(event: InputEvent): void {
		const { value } = event.target as HTMLInputElement;
		const space = match (value, {
			"local": () => TransformSpace.Local,
			"world": () => TransformSpace.World,
			_: () => {
				throw new Error(`Unknown transform space: "${value}"`);
			}
		});

		this.transformSpace = space;
		studio.setTransformSpace(space);
	}

	onGizmoToggled(event: InputEvent): void {
		const { name, checked } = event.target as HTMLInputElement;
		const flag = match (name, {
			"translate": () => TransformType.TRANSLATE,
			"rotate": () => TransformType.ROTATE,
			"scale": () => TransformType.SCALE,
			_: () => {
				throw new Error(`Unknown gizmo name: "${name}"`);
			}
		});

		if (checked) {
			this.transformType |= flag;
		} else {
			this.transformType &= ~flag;
		}

		studio.setTransformType(this.transformType);
	}

	#initStudio(): HTMLCanvasElement | null {
		const width = window.innerWidth;
		const height = window.innerHeight;
		try {
			studio.init(new ViewportSize(width, height));
		} catch (err) {
			if (!(err as Error).message?.startsWith("Using exceptions for control flow"))
				throw err;
		}

		return document.querySelector<HTMLCanvasElement>(`canvas[alt="Storyteller Studio"]`);
	}

	protected override render = () => html`
		<!-- TODO: What's a "component"? -->
		<div class="toolbar" role="toolbar">
			<div class="toolbar-group" role="group">
				<label class="btn-toggle">
					<sts-icon icon=${"cube"}>Local</sts-icon>
					<input type="radio"
						name="transform-space"
						value="local"
						?checked=${this.transformSpace === TransformSpace.Local}
						@input=${this.onSpaceChange}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="globe">Global</sts-icon>
					<input type="radio"
						name="transform-space"
						value="world"
						?checked=${this.transformSpace === TransformSpace.World}
						@input=${this.onSpaceChange}
					/>
				</label>
			</div>
			<div class="toolbar-group" role="group">
				<label class="btn-toggle">
					<sts-icon icon="up-down-left-right">Translate</sts-icon>
					<input type="checkbox"
						name="translate"
						?checked=${this.transformType & TransformType.TRANSLATE}
						@input=${this.onGizmoToggled}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="rotate">Rotate</sts-icon>
					<input type="checkbox"
						name="rotate"
						?checked=${this.transformType & TransformType.ROTATE}
						@input=${this.onGizmoToggled}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="scale">Scale</sts-icon>
					<input type="checkbox"
						name="scale"
						?checked=${this.transformType & TransformType.SCALE}
						disabled
					/>
				</label>
			</div>
		</div>

		<sts-button class="btn-help"
			icon="circle-info"
			@click=${this.toggleHelp}
		>
			Help
		</sts-button>

		${this.showHelp ? html`
			<sts-help class="help"></sts-help>
		` : null}
	`;
}
