import * as studio from "@storyteller/studio";
import { LitElement, type PropertyValues, html, nothing, unsafeCSS, TemplateResult } from "lit";
import { type Ref, createRef, ref } from "lit/directives/ref.js";
import { customElement, property, state } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { styleMap } from "lit/directives/style-map.js";
import { type ScreenshotReadyEvent } from "@storyteller/studio";
import { DialogElement } from "@storyteller/studio-ui/dialog";
import { bind, on } from "@storyteller/framework";

import { VideoStylePreview } from "./api/VideoStylePreview";

import "./style-select.element";

import styles from "./temp-video-preview.element.scss?inline";
import { STYLE_OPTIONS, type VstStyle } from "./inspector-tabs/style-tab";

enum InspectorMode {
	Properties,
	Style
}

@customElement("sts-job-status")
export class JobStatus extends LitElement {
	#JobStatusRef = createRef<DialogElement>();

	protected override render = (): TemplateResult => html`
		<sts-dialog
			${ref(this.#JobStatusRef)}
			skipWrapper
			.trigger=${ true }
			.template=${[this, html`
				<sts-dialog-base class="video-style-preview">
					Hello world
				</sts-dialog-base>
			`] as const}
		></sts-dialog>
	`
}
