import * as studio from "@storyteller/studio";
import { type EntitySelectEvent } from "@storyteller/studio";
import { drag, on } from "@storyteller/framework";
import { LitElement, PropertyValues, html, nothing, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { repeat } from "lit/directives/repeat.js";
import { styleMap } from "lit/directives/style-map.js";

import styles from "./anim-timeline.element.scss?inline";

// TODO: This is basically a [signal](https://www.solidjs.com/tutorial/introduction_signals)
//       that requires manual dependency tracking. Given that requirement, it's
//       questionable whether this is even a valuable abstraction to have.
//
//       We should either make it more obviously valuable (e.g. by coming up
//       with a way to automatically handle dependencies) and move it to
//       `@storyteller/framework`, or just get rid of it.
//
//       We could maybe add a decorator that could automatically override the
//       `update` method to invalidate the cache when dependencies are changed,
//       which would look something like:
//       ```
//       @memo(["rangeStart", "rangeEnd", "_renderWidth"])
//       secondsPerPx = new Computed(() => (this.rangeEnd - this.rangeStart) / this._renderWidth);
//       ```
//       But that feels a little obtuse and overengineered. In this particular
//       case we're probably only saving on the order of nanoseconds, so I'm
//       leaning toward just nuking the idea.
class Computed extends Number {
	#value?: number;
	#compute: () => number;

	constructor (compute: () => number) {
		const value = compute();
		super(value);
		this.#compute = compute;
	}

	override valueOf(): number {
		return this.#value ??= this.#compute();
	}

	get(): number {
		return this.valueOf();
	}

	invalidate(): void {
		this.#value = undefined;
	}
}

@customElement("sts-anim-timeline")
export class AnimTimelineElement extends LitElement {
	static override styles = unsafeCSS(styles);

	/** The time (in seconds) at the left edge of the timeline view */
	@property({ type: Number }) rangeStart = 0;
	/** The time (in seconds) at the right edge of the timeline view */
	@property({ type: Number }) rangeEnd = 5;

	/** The time (in seconds) at the beginning of the active clip range */
	@property({ type: Number }) clipStart = 0;
	/** The time (in seconds) at the end of the active clip range */
	@property({ type: Number }) clipEnd = 15;

	/** The time (in seconds) of the current play-head position */
	@property({ type: Number }) playHead = 0;
	/** Keyframe positions (in seconds) */
	@property({ attribute: false }) keyframes: Record<string, number> = {};

	@state() _selectedKeyframe: string | null = null;

	/** The width (in px) of the timeline view */
	@state() _renderWidth!: number;

	/** The offset from 0seconds of the left edge of the timeline view, in px */
	@state() _offsetX = 0;
	/** The vertical scroll offset (in px) */
	@state() _offsetY = 0;

	/**
	 * `true` when the timeline is being mouse-panned. Primarily useful for
	 * setting the cursor style to the grabby hand.
	 */
	@state() _panning = false;

	/** How many seconds are represented by each px of horizontal space. */
	pxPerSecond = new Computed(() => (this.rangeEnd - this.rangeStart) / this._renderWidth);

	#resizeObserver: ResizeObserver;

	constructor () {
		super();
		this.#resizeObserver = new ResizeObserver(() => this.#updateRenderWidth());
		this.#resizeObserver.observe(this);
	}

	override connectedCallback(): void {
		super.connectedCallback();
		this.#updateRenderWidth();
	}

	override disconnectedCallback(): void {
		super.disconnectedCallback();
		this.#resizeObserver.disconnect();
	}

	protected override update(changes: PropertyValues<this>): void {
		if (
			changes.has("rangeStart")
			|| changes.has("rangeEnd")
			|| changes.has("_renderWidth")
		) {
			this.pxPerSecond.invalidate();
		}

		super.update(changes);
	}

	@on("wheel")
	onWheel(event: WheelEvent): void {
		event.preventDefault();
		event.stopImmediatePropagation();

		if (event.ctrlKey) {
			this.#zoomTimeline(event.offsetX, event.deltaY);
		} else if (event.deltaX) {
			this.#pan(-event.deltaX);
		} else if (event.shiftKey && event.deltaY) {
			this.#pan(-event.deltaY);
		}
	}

	@on("window:entity-select")
	onEntitySelected({ detail: id }: EntitySelectEvent): void {
		if (!id) {
			this._selectedKeyframe = null;
			return;
		}

		if (Array.from(Object.keys(this.keyframes)).includes(id)) {
			this._selectedKeyframe = id;
			this.dispatchEvent(new CustomEvent("playHead-change", { detail: this.keyframes[id]! }));
		} else {
			this._selectedKeyframe = null;
		}
	}

	selectKeyframe(id: string): void {
		studio.select(id);
	}

	#zoomTimeline(origin: number, delta: number): void {
		const currentRange = this.rangeEnd - this.rangeStart;
		const deltaRange = delta * 4;

		// FIXME: When zooming the timeline, the transform origin should be
		//        centered on the mouse cursor. Currently this does not work as
		//        intended, always centering on the 00:00 timestamp instead.
		const t = ((this._offsetX + origin) * this.pxPerSecond.get()) / currentRange;
		console.log(`t: ${t}`);
		console.log(`deltaStart: ${-deltaRange * t}`);
		console.log(`deltaEnd:   ${deltaRange * (1 - t)}`);
		const rangeStart = this.rangeStart - deltaRange * t;
		const rangeEnd = this.rangeEnd + deltaRange * (1 - t);

		this.dispatchEvent(new CustomEvent("rangeStart-change", { detail: rangeStart }));
		this.dispatchEvent(new CustomEvent("rangeEnd-change", { detail: rangeEnd }));
	}

	#onDragStart(): void {
		this._panning = true;
	}

	#onDragEnd(): void {
		this._panning = false;
	}

	#onDrag(event: PointerEvent): void {
		this.#pan(event.movementX);
	}

	#pan(deltaPx: number): void {
		this._offsetX += deltaPx;

		const pxPerSecond = this.pxPerSecond.get();
		const rangeStart = this.rangeStart - deltaPx * pxPerSecond;
		const rangeEnd = this.rangeEnd - deltaPx * pxPerSecond;

		this.dispatchEvent(new CustomEvent("rangeStart-change", { detail: rangeStart }));
		this.dispatchEvent(new CustomEvent("rangeEnd-change", { detail: rangeEnd }));
	}

	#onPlayheadDrag(event: PointerEvent): void {
		const deltaSeconds = event.movementX * this.pxPerSecond.get();
		const updated = this.playHead + deltaSeconds;

		this.dispatchEvent(new CustomEvent("playHead-change", { detail: updated }));
	}

	#onKeyframeDrag(id: string, event: PointerEvent): void {
		const deltaSeconds = event.movementX * this.pxPerSecond.get();
		const updated = this.keyframes[id] + deltaSeconds;

		studio.updateKeyframe(id, updated);

		const updatedKeyframes = Object.fromEntries(
			Object
				.entries(this.keyframes)
				.map(entry => entry[0] === id
					? [id, updated]
					: entry
				)
		);
		this.dispatchEvent(new CustomEvent("keyframes-change", { detail: updatedKeyframes }));
	}

	#updateRenderWidth(): void {
		this._renderWidth = this.getBoundingClientRect().width;
	}

	/**
	 * Format a time value (in seconds) for displaying in the timeline view
	 */
	#timestamp(seconds: number) {
		const sign = seconds < 0 ? "-" : "";
		seconds = Math.abs(seconds);

		let s = Math.floor(seconds);
		let frac = Math.floor(seconds * 1000 - s * 1000);
		if (frac === 1000) {
			frac = 0;
			s += 1;
		}

		let m = 0;
		if (s >= 60) {
			m = Math.floor(s / 60);
			s %= 60;
		}

		return html`${
			sign
		}${
			m.toString(10).padStart(2, "0")
		}:${
			s.toString(10).padStart(2, "0")
		}<!--<span class="frac">.${
			frac.toString(10).padEnd(3, "0")
		}</span>-->`;
	}

	/**
	 * Convert a time value (in seconds) to a px offset from the left edge of the view
	 */
	#secondsToPx(seconds: number): number {
		return this._offsetX + seconds * this.pxPerSecond.get();
	}

	protected override render = () => {
		const pxPerSecond = this.pxPerSecond.get();
		const range = this.rangeEnd - this.rangeStart;

		const subdivsCount = Math.floor(this._renderWidth / 64);
		const pxPerSubdiv = this._renderWidth / subdivsCount;
		const secondsPerSubdiv = pxPerSecond / pxPerSubdiv;

		console.log("Seconds per Subdiv: ", secondsPerSubdiv);
		const subdivs = new Array(subdivsCount + 2).fill(null);

		const fracPxOffset = this._offsetX % pxPerSubdiv;

		let startingIdx = -Math.floor(this._offsetX / pxPerSubdiv);
		if (fracPxOffset < 0)
			startingIdx -= 2;
		else
			startingIdx -= 1;

		return html`
			<div
				class=${classMap({
					wrapper: true,
					panning: this._panning,
				})}
				${drag({
					button: 1,
					start: this.#onDragStart,
					move: this.#onDrag,
					end: this.#onDragEnd,
				})}
				tabindex="-1"
			>
				<div class="clip-region"
					style=${styleMap({
						left: `${this.#secondsToPx(this.clipStart)}px`,
						right: `${this._renderWidth - this.#secondsToPx(this.clipEnd)}px`,
					})}
				></div>

				${subdivs.map((_, idx) => {
					const pxOffset = (idx - 1) * pxPerSubdiv + fracPxOffset;
					const seconds = (startingIdx + idx) * secondsPerSubdiv;

					const secondsFrac = Math.abs(Math.round(seconds));
					const strong = secondsFrac === 0;

					return html`
						<div
							class=${classMap({
								hash: true,
								strong,
							})}
							style=${styleMap({
								left: `${pxOffset}px`
							})}
						></div>
						${strong ? html`
							<span class="timestamp"
								style=${styleMap({
									left: `${pxOffset + 8}px`
								})}
							>
								${seconds}
							</span>
						` : nothing}
					`;
				})}

				<div class="playhead"
					${drag({
						move: this.#onPlayheadDrag
					})}
					style=${styleMap({
						left: `${this.#secondsToPx(this.playHead)}px`
					})}
					tabindex="-1"
				>
					<div class="head"></div>
					<div class="bar"></div>
				</div>

				${repeat(
					Object.entries(this.keyframes),
					([id]) => id,
					([id, keyframe]) => html`
						<div id=${id}
							class=${classMap({
								keyframe: true,
								selected: this._selectedKeyframe === id,
							})}
							style=${styleMap({
								left: `${this.#secondsToPx(keyframe)}px`
							})}
							tabindex="-1"
							${drag({
								move: event => this.#onKeyframeDrag(id, event),
							})}
							@click=${() => this.selectKeyframe(id)}
						></div>
					`
				)}
			</div>
		`;
	}
}
