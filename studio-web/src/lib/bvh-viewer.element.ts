import * as studio from "@storyteller/studio";
import { ViewportSize } from "@storyteller/studio";
import { customElement, property } from "lit/decorators.js";

import { StudioBaseElement } from "./studio-base.element";
import { PropertyValues } from "lit";

@customElement("sts-bvh-viewer")
export class BvhViewerElement extends StudioBaseElement {
	@property() bvhPath?: string;
	@property() skyboxId?: string;

	onStart(viewportSize: ViewportSize): void {
		if (!this.bvhPath) return;
		if (!this.skyboxId) return;

		studio.startBvhViewer(
			this.bvhPath,
			this.skyboxId,
			`#${this._canvasProvider!.canvasId}`,
			"Storyteller Studio",
			viewportSize,
		);
	}

	onChanges(changes: PropertyValues<this>): void {
		if (changes.has("bvhPath") && !!this.bvhPath) {
			// TODO
		}

		if (changes.has("skyboxId") && !!this.skyboxId)
			studio.setSkybox(this.skyboxId);
	}
}
