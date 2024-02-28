export enum LcmLiveStatus {
	None = "NONE",
	Connected = "connected",
	Disconnected = "disconnected",
	Wait = "wait",
	SendFrame = "send_frame",
	Timeout = "timeout",
}

export interface ImageInfo {
	blob: Blob;
	prompt: string;
	negativePrompt: string;
	seed: number;
	guidanceScale: number;
}

interface LcmMessageData {
	status: LcmLiveStatus | "error";
	message?: string;
}

export class LcmClient extends WebSocket {
	#userId: string;

	get status() { return this.#status; }
	#status = LcmLiveStatus.None;

	#waiting = false;

	#blob?: Blob;
	#prompt?: string;
	#negativePrompt?: string;
	#seed?: number;
	#guidanceScale = 0.8;

	constructor (userId: string) {
		super(`wss://b3000.pancake-chimaera.ts.net/api/ws/${userId}`);
		this.#userId = userId;

		this.addEventListener("open", () => this.#onOpen());
		this.addEventListener("close", () => this.#onClose());
		this.addEventListener("message", event => this.#onMessage(event))
	}

	update(data: Partial<ImageInfo>): void {
		if ("blob" in data)
			this.#blob = data.blob;

		if ("prompt" in data)
			this.#prompt = data.prompt;

		if ("negativePrompt" in data)
			this.#negativePrompt = data.negativePrompt;

		if ("seed" in data)
			this.#seed = data.seed;

		this.#update();
	}

	#onOpen(): void {
		console.log(`Marking connection to websocket as open for user ${this.#userId}`);
		this.#status = LcmLiveStatus.Connected;
	}

	#onClose(): void {
		console.log(`Marking connection to websocket as closed for user ${this.#userId}`);
		this.#status = LcmLiveStatus.Disconnected;
	}

	#onMessage(event: MessageEvent<string>): void {
		console.log("onMessage:", event);
		const data: LcmMessageData = JSON.parse(event.data);

		if (data.status === "error") {
			this.#status = LcmLiveStatus.Disconnected;
			console.error(data.message);

			return;
		}

		this.#status = data.status;

		if (this.#status === LcmLiveStatus.SendFrame && this.#waiting)
			this.#sendFrame();
	}

	#update(): void {
		if (this.#status === LcmLiveStatus.SendFrame) {
			this.#sendFrame();
		} else {
			this.#waiting = true;
		}
	}

	#sendFrame(): void {
		if (!this.#blob || !this.#prompt || !this.#negativePrompt || this.#seed == null) {
			this.#waiting = true;
			return;
		}

		console.log("Sending next frame...");

		this.send(JSON.stringify({ status: "next_frame" }));
		this.send(JSON.stringify({
			prompt: this.#prompt,
			negativePrompt: this.#negativePrompt,
			seed: this.#seed,
			guidanceScale: this.#guidanceScale,
		}));
		this.send(this.#blob);

		this.#waiting = false;

		setTimeout(() => {
			this.dispatchEvent(new CustomEvent("lcm-image-ready", {
				detail: `https://b3000.pancake-chimaera.ts.net/api/stream/${this.#userId}`
			}));
		});
	}
}

export interface LcmClientEventMap extends WebSocketEventMap {
	"lcm-image-ready": CustomEvent<string>;
}

export interface LcmClient {
	addEventListener<K extends keyof LcmClientEventMap>(
		type: K,
		listener: (this: LcmClient, ev: LcmClientEventMap[K]) => any,
		options?: boolean | AddEventListenerOptions
	): void;
}
