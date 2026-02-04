/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly DEV: boolean;
  readonly MODE: string;
  readonly BASE_URL: string;
  readonly PROD: boolean;
  readonly SSR: boolean;
  readonly VITE_API_BASE_URL: string;
  readonly VITE_STORAGE_URL: string;
  readonly VITE_REALTIME_URL: string;
  readonly VITE_ENABLE_MOCK: string;
  readonly VITE_ENABLE_WS: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
