import * as studio from "@storyteller/studio";
import { type CameraKeyframesEvent, type SeekTimeChanged, type UpdateAnimationTargets } from "@storyteller/studio";
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
	@state() clipEnd = 15_000;

	@state() playHead = 0;
	@state() playing = false;

	@state() tracks : Record<string, {name?: string, target_type: string }> = {};
	
	@state() track_ordered : string[] = [];

	@state() keyframes : Record<string, number[]> = {};

	@state() selected_track : string | null = null;

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

		this.tracks = tracks;
	}

	onSidebarResize(event: PointerEvent): void {
		this.sidebarWidth = Math.max(Math.min(this.sidebarWidth + event.movementX, 512), 128);
	}

	addKeyframe(): void {
		studio.addCameraKeyframe(this.playHead);
	}

	jumpToStart(): void {
		this.playHead = this.clipStart;
	}

	prevKeyframe(): void {
		let pos = this.playHead;

		this.playHead = pos;
	}

	nextKeyframe(): void {
		let pos = this.playHead;

		this.playHead = pos;
	}

	jumpToEnd(): void {
		this.playHead = this.clipEnd;
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
		let track_count = {
			'--track-count': `${this.trackCount()}`,
			'--row-count': `calc(var(--track-count) + 1)`
		};
		return html`
		<div class="sidebar"
			style=${styleMap({
				width: `${this.sidebarWidth}px`,
				...track_count
			})}
		>
			<sts-toolbar>
				<sts-toolbar-group>
					<sts-button class="btn-keyframe"
						icon="key"
						aria-label="Add Keyframe"
						title="Add Keyframe"
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
			${
				this.track_ordered.map((entity) => {
					let track = this.tracks[entity];
					let name = track.name ?? track.target_type;
					let icon = TRACK_ICONS[track.target_type] ?? "minus";
					return html!`<sts-tree class="track"
						role="listitem"
						name=${name}
						icon=${icon}
					></sts-tree>`;
				})
			}
			</div>
		</div>

		<div class="divider" style=${styleMap(track_count)}
			${drag({
				move: this.onSidebarResize,
			})}
		></div>

		<sts-anim-timeline class="timeline"
		style=${styleMap(track_count)}
			.rangeStart=${bind(this, "timelineStart")}
			.rangeEnd=${bind(this, "timelineEnd")}
			.clipStart=${this.clipStart}
			.clipEnd=${this.clipEnd}
			.playHead=${bind(this, "playHead")}
			.keyframes=${bind(this, "keyframes")}
		></sts-anim-timeline>
	`
	};
};

