import * as studio from "@storyteller/studio";
import { ViewportSize } from "@storyteller/studio";
import { throttle } from "lodash";

import "./app.element.scss";

export class AppElement extends HTMLElement {
	static observedAttributes = [];

	connectedCallback(): void {
		const width = window.innerWidth;
		const height = window.innerHeight;
		try {
			studio.init(new ViewportSize(width, height));
		} catch (err) {
			if (!(err as Error).message?.startsWith("Using exceptions for control flow"))
				throw err;
		}

		window.addEventListener("resize", this.onResize);
	}

	disconnectedCallback(): void {
		window.removeEventListener("resize", this.onResize);
	}

	onResize = throttle(() => {
		const width = window.innerWidth;
		const height = window.innerHeight;
		studio.resize(new ViewportSize(width, height));
	}, 16.666);
}
customElements.define("app-root", AppElement);
