import * as studio from "@storyteller/studio";
import type { EntitySelectEvent, SeekTimeChanged, UpdateAnimationTargets } from "@storyteller/studio";
import { bind, drag, on } from "@storyteller/framework";
import { LitElement, type PropertyValues, html, unsafeCSS, TemplateResult, PropertyValueMap } from "lit";
import { customElement, state } from "lit/decorators.js";
import { styleMap } from "lit/directives/style-map.js";

import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/toolbar";
import "@storyteller/studio-ui/tree";
import "./anim-timeline.element";

import styles from "./anim-editor.element.scss?inline";

const TRACK_ICONS : Record<string, string> = {
	camera: "video",
	transform: "up-down-left-right",
	light: "light"
};

@customElement("sts-anim-editor")
export class AnimEditorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() sidebarWidth = 256;

	@state() timelineStart = 0;
	@state() timelineEnd = 5;

	@state() clipStart = 0;
	@state() clipEnd = 0;

	@state() playHead = 0;
	@state() playing = false;

	@state() tracks : Record<string, {name?: string, target_type: string }> = {};
	
	@state() track_ordered : string[] = [];

	@state() keyframes : Record<string, Keyframe> = {};

	@state() selected_track : { track: string } | {outside_entity: string } | false = false;

	trackCount(): number {
		return Object.keys(this.tracks).length;
	}

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("playHead"))
			studio.seekTime(this.playHead);

		super.update(changes);
	}

	@on("window:seek-time-changed")
	onSeekTimeChanged({ detail: {time, playing} }: SeekTimeChanged ): void {
		console.log("Seek Time Changed", time, playing);
		this.playHead = time;
		this.playing = playing;
	}

	@on("window:update-animation-targets")
	onUpdateAnimationTargets({detail: { added, removed}}: UpdateAnimationTargets) : void {
		let addedKeys = Object.keys(added);
		if (addedKeys.length === 0 && removed.length === 0) {
			return;
		}
		console.log("Animation Targets Changed");
		let tracks = {...this.tracks};
		for (let track_entity of addedKeys) {
			let track = added[track_entity];
			tracks[track_entity] = track;
		}
		
		for (let track_entity of removed) {
			delete tracks[track_entity];
		}

		this.track_ordered = Object.keys(tracks);

		if (typeof this.selected_track === "object") {
			if ("track" in this.selected_track && !(this.selected_track.track in tracks)) {
				this.selected_track = { outside_entity: this.selected_track.track};
			} else if ("outside_entity" in this.selected_track && this.selected_track.outside_entity in tracks) {
				this.selected_track = { track: this.selected_track.outside_entity};
			}
		}

		this.tracks = tracks;
	}

	@on("window:update-keyframes")
	onUpdateKeyframes({detail: {added, removed}}: studio.UpdateKeyframes): void {
		let addedKeyframes = Object.keys(added);
		if (addedKeyframes.length === 0 && removed.length === 0) {
			return;
		}
		console.log("Keyframes Changed");
		let keyframes = {...this.keyframes};
		for (let keyframe of addedKeyframes) {
			keyframes[keyframe] = added[keyframe];
		}

		this.keyframes = keyframes;

		let min = Infinity
		let max = -Infinity;
		for (const keyframe of Object.keys(this.keyframes)) {
			let time = this.keyframes[keyframe].time;
			if (typeof time != "number") {
				continue;
			}
			if (time > max) {
				max = time;
			}
			if (time < min) {
				min = time;
			}
		}
		this.clipStart = Math.max(min, 0.);
		this.clipEnd = Math.max(max, min);
	}

	
	@on("window:entity-select")
	onEntitySelected({ detail: id }: EntitySelectEvent): void {
		if (id) {
			if (this.tracks[id]) {
				this.selected_track = { track: id };
			} else if (!this.keyframes[id]) {
				this.selected_track = {outside_entity: id};
			} else {
				this.selected_track = false;
			}
		} else {
			this.selected_track = false;
		}
	}

	@on("rangeStart-change")
	onRangeStartChange({detail}: {detail:number}): void {
		this.timelineStart = detail;
	}

	
	@on("rangeEnd-change")
	onRangeEndChange({detail}: {detail:number}): void {
		this.timelineStart = detail;
	}

	onSidebarResize(event: PointerEvent): void {
		this.sidebarWidth = Math.max(Math.min(this.sidebarWidth + event.movementX, 512), 128);
	}

	addKeyframe(): void {
		if (typeof this.selected_track === "object" && "track" in this.selected_track) {
			studio.recordKeyframe([this.selected_track.track]);
		}
	}

	jumpToStart(): void {
		this.playHead = this.clipStart;
	}

	prevKeyframe(): void {
		let pos = this.playHead;

		let closest_keyframe = -1 * Infinity;

		for (const keyframe of Object.keys(this.keyframes)) {
			const time = this.keyframes[keyframe]?.time;
			if (!time || typeof time != "number") {
				continue;
			}
			if (time < pos && time > closest_keyframe ) {
				closest_keyframe = time;
			}
		}

		if (closest_keyframe < 0) {
			return;
		}

		this.playHead = closest_keyframe;
	}

	nextKeyframe(): void {

		let pos = this.playHead;

		let closest_keyframe = Infinity;

		for (const keyframe of Object.keys(this.keyframes)) {
			const time = this.keyframes[keyframe]?.time;
			if (!time || typeof time != "number") {
				continue;
			}
			if (time > pos && time < closest_keyframe ) {
				closest_keyframe = time;
			}
		}

		if (closest_keyframe > this.timelineEnd) {
			return;
		}

		this.playHead = closest_keyframe;
	}

	jumpToEnd(): void {
		this.playHead = this.clipEnd;
	}

	addTarget(): void {
		if (typeof this.selected_track === "object" && "outside_entity" in this.selected_track) {
			studio.addEntityTrack(this.selected_track.outside_entity);
		}
	}

	togglePlaying(): void {
		console.log("toggling play");
		if (this.playing) {
			studio.pause();
		} else {
			studio.play();
		}
	}

	protected override render(): TemplateResult {

		const track_order_hashed : Record<string, number> = {};

		for (const [index, track] of this.track_ordered.entries()) {
			track_order_hashed[track] = index;
		}

		const selected = typeof this.selected_track === "object" && "track" in this.selected_track ? this.selected_track.track : false;
		const can_add_selected = typeof this.selected_track === "object" && "outside_entity" in this.selected_track ? this.selected_track.outside_entity : false;

		let length = this.timelineEnd - this.timelineStart;
		let clip_start = (this.clipStart - this.timelineStart) / length;
		let clip_end = (this.clipEnd - this.timelineStart) / length;

		
		let timeline_css_variables = {
			'--track-count': `${this.trackCount()}`,
			'--row-count': `calc(var(--track-count) + ${can_add_selected ? 2 : 1})`,
			'--clip-end': `${clip_end * 100}%`,
			'--clip-start': `${clip_start * 100}%`
		};

		const tracklist = this.track_ordered.map((entity) => {
			let track = this.tracks[entity];
			let name = track.name ?? track.target_type;
			let icon = TRACK_ICONS[track.target_type] ?? "minus";
			return html`
			<sts-tree class=${`track ` + (selected === entity ? `selected` : ``)}
				role="listitem"
				name=${name}
				icon=${icon}
				@click=${() => {
					studio.select(entity);
				}}
			></sts-tree>
			`;
		});

		if (can_add_selected) {
			tracklist.push(html`
				<sts-tree class=${`track`}
				role="button"
				name="animate current selection"
				icon="plus-large"
				@click=${this.addTarget}></sts-tree>
			`);
		}

		const none_selected = !selected;

		return html`
		<div class="sidebar"
			style=${styleMap({
				width: `${this.sidebarWidth}px`,
				...timeline_css_variables
			})}
		>
			<sts-toolbar>
				<sts-toolbar-group>
					<sts-button class="btn-keyframe"
						icon="key"
						aria-label="Add Keyframe"
						title="Add Keyframe"
						?disabled=${none_selected}
						@click=${this.addKeyframe}
					></sts-button>
				</sts-toolbar-group>
				<sts-toolbar-group class="timeline-nav">
					<sts-button
						icon="bwd-fast"
						aria-label="Jump to Start"
						title="Jump to Start"
						@click=${this.jumpToStart}
					></sts-button>
					<sts-button
						icon="bwd-step"
						aria-label="Previous Keyframe"
						title="Previous Keyframe"
						@click=${this.prevKeyframe}
					></sts-button>
					<sts-button class="play"
						icon=${this.playing ? "pause" : "play" }
						aria-label="Play"
						title="Play"
						@click=${this.togglePlaying}
					></sts-button>
					<sts-button
						icon="fwd-step"
						aria-label="Next Keyframe"
						title="Next Keyframe"
						@click=${this.nextKeyframe}
					></sts-button>
					<sts-button
						icon="fwd-fast"
						aria-label="Jump to End"
						title="Jump to End"
						@click=${this.jumpToEnd}
					></sts-button>
				</sts-toolbar-group>
			</sts-toolbar>
			<div class="tracklist" role="list">
			${tracklist}
			</div>
		</div>

		<div class="divider" style=${styleMap(timeline_css_variables)}
			${drag({
				move: this.onSidebarResize,
			})}
		></div>

		<sts-anim-timeline class="timeline"
		style=${styleMap(timeline_css_variables)}
			.rangeStart=${bind(this, "timelineStart")}
			.rangeEnd=${bind(this, "timelineEnd")}
			.clipStart=${this.clipStart}
			.clipEnd=${this.clipEnd}
			.playHead=${bind(this, "playHead")}
			.keyframes=${bind(this, "keyframes")}
			.track_ordered=${track_order_hashed}
		></sts-anim-timeline>
	`
	};
};

