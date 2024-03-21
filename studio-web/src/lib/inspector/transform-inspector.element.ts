import * as studio from "@storyteller/studio"
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { type Quat, type Vec3 } from "./inspector.types";

import styles from "./transform-inspector.element.scss?inline";

@customElement("sts-transform-inspector")
export class TransformInspectorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ attribute: false })
	translation: Vec3 = { x: 0, y: 0, z: 0 };

	@property({ attribute: false })
	rotation: Quat = { w: 1, x: 0, y: 0, z: 0 };

	@property({ attribute: false })
	scale: Vec3 = { x: 1, y: 1, z: 1 };

	updateField(reflectPath: `${"translation"|"rotation"|"scale"}.${"x"|"y"|"z"}`) {
		return ({ detail }: CustomEvent<string>) => {
			const value = parseFloat(detail);
			if (Number.isFinite(value) && !Number.isNaN(value)) {
				studio.updateComponentField({
					component: "bevy_transform::components::transform::Transform",
					reflectPath,
					value,
				});
			}
		}
	}

	protected override render = () => html`
		<h4>Transform</h4>
		<div class="fieldset" role="group">
			<legend>Translation</legend>
			<sts-text-field
				type="number"
				.value=${this.translation.x.toFixed(3)}
				@value-change=${this.updateField("translation.x")}
			></sts-text-field>
			<sts-text-field
				type="number"
				.value=${this.translation.y.toFixed(3)}
				@value-change=${this.updateField("translation.y")}
			></sts-text-field>
			<sts-text-field
				type="number"
				.value=${this.translation.z.toFixed(3)}
				@value-change=${this.updateField("translation.z")}
			></sts-text-field>
		</div>
		<div class="fieldset" role="group">
			<legend>Rotation</legend>
			<!-- TODO: This is not how quaternions work lol -->
			<sts-text-field
				type="number"
				.value=${this.rotation.x.toFixed(3)}
				@value-change=${this.updateField("rotation.x")}
			></sts-text-field>
			<sts-text-field
				type="number"
				.value=${this.rotation.y.toFixed(3)}
				@value-change=${this.updateField("rotation.y")}
			></sts-text-field>
			<sts-text-field
				type="number"
				.value=${this.rotation.z.toFixed(3)}
				@value-change=${this.updateField("rotation.z")}
			></sts-text-field>
			<!-- <input type="number" value=${this.rotation.w.toFixed(3)} /> -->
		</div>
		<div class="fieldset" role="group">
			<legend>Scale</legend>
			<sts-text-field
				type="number"
				.value=${this.scale.x.toFixed(3)}
				@value-change=${this.updateField("scale.x")}
			></sts-text-field>
			<sts-text-field
				type="number"
				.value=${this.scale.y.toFixed(3)}
				@value-change=${this.updateField("scale.y")}
			></sts-text-field>
			<sts-text-field
				type="number"
				.value=${this.scale.z.toFixed(3)}
				@value-change=${this.updateField("scale.z")}
			></sts-text-field>
		</div>
	`;
}
