import * as studio from "@storyteller/studio";
import { match } from "@storyteller/utility";
import { LitElement, type PropertyValues, html, nothing, unsafeCSS, TemplateResult } from "lit";
import { type Ref, createRef, ref } from "lit/directives/ref.js";
import { customElement, property, state } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { styleMap } from "lit/directives/style-map.js";
import { type ScreenshotReadyEvent } from "@storyteller/studio";
import { DialogElement } from "@storyteller/studio-ui/dialog";
import { bind, on } from "@storyteller/framework";

import { VideoStylePreview } from "./api/VideoStylePreview";
import { STYLE_OPTIONS, type VstStyle } from "./inspector-tabs/style-tab";
import { CompositorJobStatus } from "./style-workspace.element";

import "./style-select.element";

import styles from "./temp-video-preview.element.scss?inline";

enum InspectorMode {
	Properties,
	Style
}

@customElement("sts-job-status-dialog")
export class JobStatusDialogElement extends LitElement {
	@property() status?: CompositorJobStatus;

	protected override render = (): TemplateResult => html`
		<sts-dialog-base class="video-style-preview">
			<h1>
				${match(this.status, {
					[CompositorJobStatus.Success]: () => html`:D`,
					[CompositorJobStatus.Failure]: () => html`D:`,
					_: () => html`¯\_(ツ)_/¯`,
				})}
			</h1>
		</sts-dialog-base>
	`
}
