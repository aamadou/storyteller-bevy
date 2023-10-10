import { match } from "@storyteller/utility";
import { ElementPart, LitElement, html, nothing, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Directive, directive } from "lit/directive.js";
import { unsafeSVG } from "lit/directives/unsafe-svg.js";

import fasCircleInfo from "@fortawesome/fontawesome-pro/svgs/solid/circle-info.svg?raw";
import fasCube from "@fortawesome/fontawesome-pro/svgs/solid/cube.svg?raw";
import fasGlobe from "@fortawesome/fontawesome-pro/svgs/solid/globe.svg?raw";
import fasUpDownLeftRight from "@fortawesome/fontawesome-pro/svgs/solid/up-down-left-right.svg?raw";
import fasRotate from "@fortawesome/fontawesome-pro/svgs/solid/rotate.svg?raw";
import fasScale from "@fortawesome/fontawesome-pro/svgs/solid/arrow-up-right-and-arrow-down-left-from-center.svg?raw";

import styles from "./icon.element.scss?inline";

@customElement("sts-icon")
export class IconElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() icon = "";

	protected override render = () => html`
		<span class="a11y-label">
			<slot></slot>
		</span>
		<div class="svg-wrapper" role="presentation">
			${match (this.icon, {
				"circle-info": () => renderSvg(fasCircleInfo),
				"cube": () => renderSvg(fasCube),
				"globe": () => renderSvg(fasGlobe),
				"up-down-left-right": () => renderSvg(fasUpDownLeftRight),
				"rotate": () => renderSvg(fasRotate),
				"scale": () => renderSvg(fasScale),
				_: () => nothing,
			})}
		</div>
	`;
}

// TODO: Consider moving this
function renderSvg(fullSvg: string) {
	console.log("parsing:", fullSvg);

	const [, attrsString, body] = fullSvg.match(/<svg(.+?)>(.+?)<\/svg>/)!;
	const attrs = Object.fromEntries(
		Array.from(attrsString.matchAll(/([-:_a-zA-Z0-9]+)(?:=['"]?([^'"]*)['"]?)/g))
			.map(([, attr, value]) => [attr, value])
	);

	return html`
		<svg ${spread(attrs)}>
			${unsafeSVG(body)}
		</svg>
	`;
}

// TODO: Definitely move this
const spread = directive(class extends Directive {
	override render(_: Record<string, string>) {
		return nothing;
	}

	override update(part: ElementPart, [attrs]: [Record<string, string>]) {
		for (let [key, value] of Object.entries(attrs)) {
			part.element.setAttribute(key, value);
		}
		return part;
	}
});
