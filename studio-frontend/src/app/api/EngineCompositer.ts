import MakeRequest from "./MakeRequest";

export interface EngineCompositerRequest {
  uuid_idempotency_token: string,
  media_file_token: string,
  camera?: string,
  camera_speed?: number,
  skybox?: string,
}

export interface EngineCompositerResponse {
  success: boolean,
  inference_job_token?: string
}

export const EngineCompositer = MakeRequest<string, EngineCompositerRequest, EngineCompositerResponse,{}>({
  method: "POST",
  routingFunction: () => `/v1/conversion/enqueue_render_engine_scene`,
});