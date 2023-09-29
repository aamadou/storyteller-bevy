import * as helloWorld from "@storyteller/hello-world";

import "./app.element.scss";

export class AppElement extends HTMLElement {
	static observedAttributes = [];

	connectedCallback(): void {
		// TODO: This siezes up the whole browser for several seconds, presumably
		// while the WASM binary is loading. Is there any way to do this
		// asynchronously?
		// TODO: Since we're using `#[wasm_bindgen]` attributes instead of the
		// CLI, we can craft our own JS API to interact with Bevy! \o/ As a start,
		// we should exploit this to pass an explicit canvas size to Bevy and keep
		// it up to date with the browser viewport size.
		helloWorld.init();
	}
}
customElements.define("app-root", AppElement);
