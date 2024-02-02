import { on } from "@storyteller/framework";
import { type InspectorDomChanged } from "dist/studio/studio";
import { LitElement, html, unsafeCSS, nothing, HTMLTemplateResult } from "lit";
import { customElement, state } from "lit/decorators.js";

@customElement("sts-inspector")
export class AppElement extends LitElement {
    @state() vdom?: string | null;

    @on("window:inspector-dom-changed")
	onInspectorDomChanged({ detail: state}: InspectorDomChanged): void {
		console.log("inspector dom changed:", state);
	}

    protected override render = () => html` 
        <div id="some_id">

        </div>
    `;

}