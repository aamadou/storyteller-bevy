import axios from "axios";

export type MediaToken = string;

export class StorytellerApi {
	#api = axios.create({
		baseURL: "https://api.fakeyou.com",
		withCredentials: true,
	});

	async createScene(file: File): Promise<MediaToken> {
		type Response = {
			success: boolean;
			media_file_token: MediaToken;
		}

		const response = await this.#api.postForm<Response>("v1/engine/create_scene", {
			uuid_idempotency_token: crypto.randomUUID(),
			file,
		}, {
			headers: { Accept: "text/plain" },
		});

		if (response.status === 200 && response.data.success)
			return response.data.media_file_token;

		throw new Error(`API responded with status ${response.status}: "${response.statusText}"`);
	}
}
