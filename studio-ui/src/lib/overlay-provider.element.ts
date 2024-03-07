import { UniqueToken, provide } from "@storyteller/framework";
import { LitElement, TemplateResult, html, render } from "lit";
import { customElement } from "lit/decorators.js";
import { Ref, createRef, ref } from "lit/directives/ref.js";
import { styleMap } from "lit/directives/style-map.js";

export const OVERLAY_PROVIDER = UniqueToken.create<OverlayProvider>();

// TODO: Document this
export interface OverlayProvider {
	attach(host: LitElement, template: TemplateResult): WeakRef<OverlayHandle>;
	detach(handle: WeakRef<OverlayHandle>): void;
	reRender(handle: WeakRef<OverlayHandle>): void;
}


// TODO: Document this
export interface OverlayHandle {
	readonly id: number;
}

class OverlayHandleInternal implements OverlayHandle {
	#id: number;
	get id() { return this.#id; }

	constructor (id: number) {
		this.#id = id;
	}
	_updateId(id: number): void {
		this.#id = id;
	}
}

@customElement("sts-overlay-provider")
@provide(OVERLAY_PROVIDER)
export class OverlayProviderElement
	extends LitElement
	implements OverlayProvider
{
	#handles: OverlayHandleInternal[] = [];
	#containerRefs: Ref<HTMLElement>[] = [];
	#containerTemplates: TemplateResult[] = [];
	#templates: [LitElement, TemplateResult][] = [];

	attach(host: LitElement, template: TemplateResult): WeakRef<OverlayHandle> {
		const index = this.#templates.length;
		const handle = new OverlayHandleInternal(index);

		this.#handles.push(handle);
		this.#containerRefs.push(createRef());
		this.#containerTemplates.push(html`
			<div
				${ref(this.#containerRefs[index])}
				slot="overlay"
				style=${styleMap({
					position: "fixed",
					inset: 0,
				})}
			></div>
		`);
		this.#templates.push([host, template]);

		this.#updateLightDom();

		return new WeakRef(handle);
	}

	detach(handle: WeakRef<OverlayHandle>): void {
		const index = handle.deref()?.id;
		if (index == null) return;

		this.#handles.splice(index, 1);
		this.#containerRefs.splice(index, 1);
		this.#containerTemplates.splice(index, 1);
		this.#templates.splice(index, 1);

		for (let i = index; i < this.#handles.length; ++i)
			this.#handles[i]._updateId(i);

		this.#updateLightDom();
	}

	reRender(handle: WeakRef<OverlayHandle>): void {
		const index = handle.deref()?.id;
		if (index == null) return;

		const container = this.#containerRefs[index]?.value;
		if (!container) return;

		const tuple = this.#templates[index];
		if (!tuple) return;

		const [host, template] = tuple;
		render(template, container, { host });
	}

	#updateLightDom(): void {
		render(this.#containerTemplates, this, { host: this });

		for (let i = 0; i < this.#handles.length; ++i) {
			const container = this.#containerRefs[i].value;
			if (!container) throw new Error("Attempted to retrieve an invalid container ref!");

			const [host, template] = this.#templates[i];
			render(template, container, { host });
		}
	}

	protected override render = () => html`
		<slot></slot>
		<slot name="overlay"></slot>
	`;
}
