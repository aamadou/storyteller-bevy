import * as studio from "@storyteller/studio";
import { ViewportSize } from "@storyteller/studio";

import "./app.element.scss";

export class AppElement extends HTMLElement {
	static observedAttributes = [];

	connectedCallback(): void {
		// TODO: This siezes up the whole browser for several seconds, presumably
		// while the WASM binary is loading. Is there any way to do this
		// asynchronously?
		const { width, height } = document.body.getBoundingClientRect();
		studio.init(new ViewportSize(width, height));
	}
}
customElements.define("app-root", AppElement);
