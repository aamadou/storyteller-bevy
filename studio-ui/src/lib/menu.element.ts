import { inject, on } from "@storyteller/framework";
import { Fn } from "@storyteller/utility";
import { LitElement, PropertyValues, css, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { createRef, ref, type Ref } from "lit/directives/ref.js";
import { styleMap } from "lit/directives/style-map.js";

import panelStyles from "./menu.element.scss?inline";

import {
	OVERLAY_PROVIDER,
	type OverlayHandle,
	type OverlayProvider,
} from "./overlay-provider.element";

@customElement("sts-menu")
export class MenuElement extends LitElement {
	static override styles = css`
		:host {
			visibility: hidden;
		}
	`;

	@property({ attribute: false })
	trigger: Ref<HTMLElement> = createRef();

	/** Close the menu when one of its menuitems is pressed */
	@property({ type: Boolean })
	autoClose = false;

	@inject(OVERLAY_PROVIDER)
	overlay!: OverlayProvider;

	#menuId = crypto.randomUUID();
	#slotRef = createRef<HTMLSlotElement>();
	#panelRef = createRef<MenuPanelElement>();
	#handle?: WeakRef<OverlayHandle>;
	#listener?: Fn;

	protected override update(changes: PropertyValues<this>): void {
		super.update(changes);

		if (this.#handle)
			this.overlay.reRender(this.#handle);
	}

	protected override updated(changes: PropertyValues<this>): void {
		if (changes.has("trigger") && this.trigger.value) {
			const trigger = this.trigger.value;

			trigger.setAttribute("aria-owns", this.#menuId);
			trigger.ariaHasPopup = "true";
			trigger.ariaExpanded = "false";

			if (this.#listener)
				trigger.removeEventListener("click", this.#listener);

			this.#listener = () => this.show();
			trigger.addEventListener("click", this.#listener);
		}

		super.updated(changes);
	}

	@on("window:click")
	onWindowClick(event: PointerEvent): void {
		if (!this.#handle || !this.#panelRef.value)
			return;

		const path = event.composedPath();
		if (
			!path.includes(this.#panelRef.value)
			&& (!this.trigger.value || !path.includes(this.trigger.value))
		) {
			this.hide();
		}
	}

	onMenuClick(event: PointerEvent): void {
		event.stopPropagation();

		if (
			this.autoClose
			&& event.target
			&& event.target instanceof HTMLElement
			&& event.target.role === "menuitem"
		) {
			this.hide();
		}
	}

	show(): void {
		const trigger = this.trigger.value;
		if (!trigger) return;

		trigger.classList.toggle("active", true);
		trigger.ariaExpanded = "true";

		const rect = trigger.getBoundingClientRect();

		this.#handle = this.overlay.attach(this, html`
			<sts-menu-panel
				${ref(this.#panelRef)}
				id=${this.#menuId}
				class=${ifDefined(this.className)}
				style=${styleMap({
					position: "absolute",
					top: `${rect.bottom + 4}px`,
					left: `${rect.left}px`,
				})}
				@click=${this.onMenuClick}
			>
				${
					// Move the nodes assigned to the <sts-menu>'s slot into the
					// <sts-menu-panel>'s slot
					this.#slotRef.value?.assignedNodes()
				}
			</sts-menu-panel>
		`);
	}

	hide(): void {
		const trigger = this.trigger.value;
		if (trigger) {
			trigger.classList.toggle("active", false);
			trigger.ariaExpanded = "false";
		}

		if (this.#panelRef.value) {
			// Move the nodes assigned to the <sts-menu-panel>'s slot back into the
			// <sts-menu>'s slot
			for (let node of this.#panelRef.value.getSlot()!.assignedNodes())
				this.appendChild(node);
		}

		if (this.#handle)
			this.overlay.detach(this.#handle);

		this.#handle = undefined;
	}

	protected override render = () => html`
		<slot ${ref(this.#slotRef)}></slot>
	`
}

@customElement("sts-menu-panel")
export class MenuPanelElement extends LitElement {
	static override styles = unsafeCSS(panelStyles);

	@property({ reflect: true })
	override role = "menu";

	#slotRef = createRef<HTMLSlotElement>();
	getSlot(): HTMLSlotElement | undefined { return this.#slotRef.value; }

	protected override render = () => html`
		<slot ${ref(this.#slotRef)}></slot>
	`
}

declare global {
	interface HTMLElementTagNameMap {
		"sts-menu-panel": MenuPanelElement;
	}
}
