import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import { TransformSpace, ViewportSize } from "@storyteller/studio";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";

import "./button.element";
import "./help.element";
import "./icon.element";

import styles from "./app.element.scss?inline";
import { match } from "@storyteller/utility";

// FIXME: Figure out how to generate the correct code from `wasm_bindgen`
enum InteractionTransform {
	TRANSLATE = 0b0001,
	ROTATE    = 0b0010,
}

@customElement("app-root")
export class AppElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() showHelp = false;

	@state() transformSpace: "local"|"world" = "local";

	@state() translateGizmoEnabled = true;
	@state() rotateGizmoEnabled = true;
	@state() scaleGizmoEnabled = false;

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

	toggleHelp(): void {
		this.showHelp = !this.showHelp;
	}

	onSpaceChange(event: InputEvent): void {
		const { value } = event.target as HTMLInputElement;
		if (value === this.transformSpace)
			return;

		match (value, {
			"local": () => studio.setTransformSpace(TransformSpace.Local),
			"world": () => studio.setTransformSpace(TransformSpace.World),
			_: () => {
				console.warn(`Unknown transform space: "${value}"`);
				return;
			}
		});

		this.transformSpace = value as "local"|"world";
	}

	onGizmoToggled(event: InputEvent): void {
		const { name, checked } = event.target as HTMLInputElement;
		match (name, {
			"translate": () => {
				if (this.translateGizmoEnabled === checked)
					return;

				this.translateGizmoEnabled = checked;
			},
			"rotate": () => {
				if (this.rotateGizmoEnabled === checked)
					return;

				this.rotateGizmoEnabled = checked;
			},
			"scale": () => {
				if (this.scaleGizmoEnabled === checked)
					return;

				this.scaleGizmoEnabled = checked;
			},
			_: () => {
				console.warn(`Unknown gizmo name: "${name}"`);
				return;
			}
		});

		let transform = 0;
		if (this.translateGizmoEnabled) transform |= InteractionTransform.TRANSLATE;
		if (this.rotateGizmoEnabled) transform |= InteractionTransform.ROTATE;
		// TODO: Scale

		studio.setInteractionTransform(transform);
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
						checked
						@input=${this.onSpaceChange}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="globe">Global</sts-icon>
					<input type="radio"
						name="transform-space"
						value="world"
						@input=${this.onSpaceChange}
					/>
				</label>
			</div>
			<div class="toolbar-group" role="group">
				<label class="btn-toggle">
					<sts-icon icon="up-down-left-right">Translate</sts-icon>
					<input type="checkbox"
						name="translate"
						checked
						@input=${this.onGizmoToggled}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="rotate">Rotate</sts-icon>
					<input type="checkbox"
						name="rotate"
						checked
						@input=${this.onGizmoToggled}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="scale">Scale</sts-icon>
					<input type="checkbox"
						name="scale"
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
