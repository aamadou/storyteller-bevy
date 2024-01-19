import * as studio from "@storyteller/studio";
import { StudioMode, ViewportSize } from "@storyteller/studio";
import { PropertyValues } from "lit";
import { customElement, property } from "lit/decorators.js";

import { StudioBaseElement } from "./studio-base.element";

@customElement("sts-viewer")
export class ViewerElement extends StudioBaseElement {
	@property() objectId?: string;
	@property() skyboxId?: string;
	@property({ type: Number }) mode?: StudioMode;

	onStart(viewportSize: ViewportSize): void {
		if (!this.objectId) return;
		if (!this.skyboxId) return;
		if (this.mode == null) return;

		studio.start(
			this.objectId,
			this.skyboxId,
			`#${this._canvasProvider!.canvasId}`,
			"Storyteller Studio",
			viewportSize,
			this.mode,
		);
	}

	onChanges(changes: PropertyValues<this>): void {
		if (changes.has("objectId") && !!this.objectId)
			studio.loadObject(this.objectId);

		if (changes.has("skyboxId") && !!this.skyboxId)
			studio.setSkybox(this.skyboxId);
	}
}

declare global {
	export interface HTMLElementTagNameMap {
		"sts-viewer": ViewerElement;
	}
}
